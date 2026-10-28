//! First-login enrollment for an explicitly selected but unbound `bw:` source.
//! Secrets only travel over stdin/stdout pipes. The recovery journal holds IDs,
//! never passwords; ambiguous create outcomes are reconciled, never replayed.
use crate::{runtime::env::Env, ssh_config::model::SshConfigFile};
use anyhow::{Context, Result, bail, ensure};
use base64::Engine;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::Write, path::Path, process::Stdio};

trait Vault {
    fn call(&self, args: &[&str], body: Option<&Value>) -> Result<Value>;
}
struct Cli<'a> {
    env: &'a Env,
    session: Option<&'a str>,
}
impl Vault for Cli<'_> {
    fn call(&self, args: &[&str], body: Option<&Value>) -> Result<Value> {
        let mut command = self.env.command("bw");
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(token) = self.session {
            command.env("BW_SESSION", token);
        }
        let mut child = command
            .spawn()
            .context("无法启动 Bitwarden CLI，请先安装 bw")?;
        if let Some(body) = body {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(body)?);
            child
                .stdin
                .take()
                .context("Bitwarden stdin unavailable")?
                .write_all(encoded.as_bytes())?;
        } else {
            drop(child.stdin.take());
        }
        let output = child.wait_with_output()?;
        // Never log CLI output: a create response includes the password.
        ensure!(
            output.status.success(),
            "Bitwarden {} 失败（退出码 {:?}）；未写入主机绑定。请检查 CLI 登录、解锁及网络。",
            args.join(" "),
            output.status.code()
        );
        if args == ["sync"] {
            return Ok(Value::Null);
        }
        serde_json::from_slice(&output.stdout)
            .context("Bitwarden 返回的数据无效；请核对密码库后重试")
    }
}
fn id(value: &Value) -> Result<String> {
    let id = value["id"].as_str().context("Bitwarden 未返回条目 ID")?;
    ensure!(
        id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'),
        "Bitwarden ID 无效"
    );
    Ok(id.to_string())
}
fn journal_write(path: &Path, identity: &str, stage: &str, id: Option<&str>) -> Result<()> {
    crate::fs_util::write_json_pretty(path, &json!({"identity":identity,"stage":stage,"id":id}))?;
    Ok(())
}
fn enroll(
    vault: &dyn Vault,
    config_path: &Path,
    alias: &str,
    journal: &Path,
    prompt: impl FnOnce() -> Result<Option<String>>,
) -> Result<Option<String>> {
    ensure!(
        !crate::demo_flag::is_demo(),
        "演示模式不保存 Bitwarden 密码"
    );
    let initial = std::fs::read(config_path)?;
    let mut config = SshConfigFile::parse(config_path)?;
    let host = config
        .host_entries()
        .into_iter()
        .find(|h| h.alias == alias)
        .context("主机已不存在")?;
    ensure!(
        host.askpass.as_deref().is_none_or(|s| s.trim() == "bw:"),
        "密码来源已改变，请重新打开主机"
    );
    let identity = serde_json::to_string(&json!([alias, host.hostname, host.user, host.port]))?;
    if let Some(parent) = journal.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _lock = crate::fs_util::FileLock::acquire(&journal.with_file_name("operations"))?;
    let recovery: Option<Value> = if journal.exists() {
        Some(serde_json::from_slice(&std::fs::read(journal)?)?)
    } else {
        None
    };
    if let Some(ref old) = recovery {
        ensure!(
            old["identity"] == identity,
            "主机身份已改变，请先核对 Bitwarden 中的已有记录"
        );
    }
    ensure!(
        vault.call(&["status"], None)?["status"] == "unlocked",
        "Bitwarden 尚未解锁；请先 bw login，再重新连接解锁密码库"
    );
    vault.call(&["sync"], None)?;
    let items = vault.call(&["list", "items", "--search", alias], None)?;
    let items = items.as_array().context("Bitwarden 项目列表无效")?;
    let matches: Vec<_> = items
        .iter()
        .filter(|i| {
            i["deletedDate"].is_null()
                && i["fields"].as_array().is_some_and(|fields| {
                    fields
                        .iter()
                        .any(|f| f["name"] == "purple_host_identity" && f["value"] == identity)
                })
        })
        .collect();
    ensure!(
        matches.len() <= 1,
        "发现多个对应的 Bitwarden 条目，请先处理重复记录"
    );
    let item_id = if let Some(item) = matches.first() {
        ensure!(
            item["login"]["username"] == host.user
                && item["login"]["password"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty()),
            "已有条目的用户名或密码不完整，请在 Bitwarden 中检查"
        );
        id(item)?
    } else if recovery
        .as_ref()
        .is_some_and(|r| r["stage"] != "creating_folder")
    {
        bail!(
            "上次创建结果尚未确认，已停止以避免重复创建。请同步并核对 Bitwarden 的 ssh/PPL 文件夹后重试。"
        )
    } else {
        if recovery.is_some() {
            let folders = vault.call(&["list", "folders"], None)?;
            ensure!(
                folders.as_array().is_some_and(|f| f
                    .iter()
                    .filter(|x| x["name"] == "ssh/PPL")
                    .count()
                    == 1),
                "上次文件夹创建结果尚未确认，请先核对 Bitwarden，避免重复创建"
            );
            std::fs::remove_file(journal)?;
        }
        ensure!(
            !host.user.is_empty() && !host.hostname.is_empty(),
            "请先填写主机地址和用户名，再保存 Bitwarden 密码"
        );
        let name = format!("PPL / {alias}");
        ensure!(
            !items.iter().any(|i| i["deletedDate"].is_null()
                && i["name"].as_str().is_some_and(|n| n.trim() == name)),
            "Bitwarden 已有同名条目，请选择其 UUID 绑定，避免覆盖密码"
        );
        let Some(password) = prompt()? else {
            return Ok(None);
        };
        ensure!(!password.is_empty(), "未输入密码，已取消保存");
        let folders = vault.call(&["list", "folders"], None)?;
        let folders: Vec<_> = folders
            .as_array()
            .context("文件夹列表无效")?
            .iter()
            .filter(|f| f["name"] == "ssh/PPL")
            .collect();
        ensure!(
            folders.len() <= 1,
            "存在多个 ssh/PPL 文件夹，请先处理重复文件夹"
        );
        let folder_id = if let Some(folder) = folders.first() {
            id(folder)?
        } else {
            journal_write(journal, &identity, "creating_folder", None)?;
            let folder = vault.call(&["create", "folder"], Some(&json!({"name":"ssh/PPL"})))?;
            let folder_id = id(&folder)?;
            // Folder was confirmed, so no unknown operation remains.
            std::fs::remove_file(journal)?;
            folder_id
        };
        journal_write(journal, &identity, "creating_item", None)?;
        let item = vault.call(&["create","item"],Some(&json!({
            "type":1,"name":name,"folderId":folder_id,"organizationId":null,"favorite":false,"reprompt":0,
            "notes":"Created by Purple on first password login.",
            "fields":[{"name":"purple_host_identity","value":identity,"type":0}],
            "login":{"username":host.user,"password":password,"totp":null,"uris":[{"uri":format!("ssh://{}:{}",host.hostname,host.port),"match":null}]}
        })))?;
        let item_id = id(&item)?;
        journal_write(journal, &identity, "created", Some(&item_id))?;
        item_id
    };
    // Confirm persistence and password presence before installing the binding.
    vault.call(&["sync"], None)?;
    let item = vault.call(&["get", "item", &item_id], None)?;
    ensure!(
        id(&item)? == item_id
            && item["login"]["password"]
                .as_str()
                .is_some_and(|p| !p.is_empty()),
        "Bitwarden 条目保存验证失败"
    );
    ensure!(
        std::fs::read(config_path)? == initial,
        "SSH 配置已被其他程序修改；Bitwarden 条目已保留，请重新连接以完成绑定"
    );
    let source = format!("bw:{item_id}");
    ensure!(
        config.set_host_askpass(alias, &source),
        "未能回填主机密码来源；密码库条目已保留"
    );
    config.write()?;
    let saved = SshConfigFile::parse(config_path)?
        .host_entries()
        .into_iter()
        .find(|h| h.alias == alias);
    ensure!(
        saved.as_ref().and_then(|h| h.askpass.as_deref()) == Some(source.as_str()),
        "配置回填验证失败；Bitwarden 条目已保留"
    );
    if journal.exists() {
        std::fs::remove_file(journal)?;
    }
    Ok(Some(source))
}
/// Called only from interactive connection paths, after the normal vault unlock.
pub fn prepare(
    env: &Env,
    config: &Path,
    alias: &str,
    source: &mut Option<String>,
    session: Option<&str>,
) -> Result<bool> {
    if !source.as_deref().is_some_and(|s| s.trim() == "bw:") {
        return Ok(true);
    }
    let paths = env.paths().context("找不到 Purple 配置目录")?;
    let key = format!(
        "{:x}",
        Sha256::digest(format!("{}:{alias}", config.display()).as_bytes())
    );
    let journal = paths
        .state_dir()
        .join("bitwarden-enrollment")
        .join(format!("{key}.json"));
    let vault = Cli { env, session };
    let result = enroll(&vault, config, alias, &journal, || {
        eprintln!("将为 {alias} 在 Bitwarden 的 ssh/PPL 文件夹保存登录密码。");
        crate::cli::prompt_hidden_input("服务器登录密码（Esc 取消）：")
    })?;
    if let Some(binding) = result {
        *source = Some(binding);
        eprintln!("Bitwarden 密码已保存并绑定，正在连接…");
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    const ITEM: &str = "11111111-1111-4111-8111-111111111111";
    const FOLDER: &str = "22222222-2222-4222-8222-222222222222";
    struct Fake {
        replies: RefCell<VecDeque<(&'static str, Result<Value>)>>,
        bodies: RefCell<Vec<Value>>,
    }
    impl Fake {
        fn new(replies: Vec<(&'static str, Result<Value>)>) -> Self {
            Self {
                replies: RefCell::new(replies.into()),
                bodies: RefCell::new(vec![]),
            }
        }
    }
    impl Vault for Fake {
        fn call(&self, args: &[&str], body: Option<&Value>) -> Result<Value> {
            let (expected, result) = self
                .replies
                .borrow_mut()
                .pop_front()
                .expect("unexpected vault call");
            assert_eq!(args.join(" "), expected);
            if let Some(b) = body {
                self.bodies.borrow_mut().push(b.clone());
            }
            result
        }
    }
    fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        std::fs::write(
            &config,
            "Host test\n  HostName example.test\n  User root\n  # purple:askpass bw:\n",
        )
        .unwrap();
        let journal = dir.path().join("journal.json");
        (dir, config, journal)
    }
    fn initial(items: Value) -> Vec<(&'static str, Result<Value>)> {
        vec![
            ("status", Ok(json!({"status":"unlocked"}))),
            ("sync", Ok(Value::Null)),
            ("list items --search test", Ok(items)),
        ]
    }
    fn item() -> Value {
        json!({"id":ITEM,"name":"PPL / test","login":{"username":"root","password":"secret-test-value"},"fields":[{"name":"purple_host_identity","value":"[\"test\",\"example.test\",\"root\",22]"}]})
    }
    fn create_steps() -> Vec<(&'static str, Result<Value>)> {
        let mut steps = initial(json!([]));
        steps.extend([
            ("list folders", Ok(json!([{"id":FOLDER,"name":"ssh/PPL"}]))),
            ("create item", Ok(item())),
            ("sync", Ok(Value::Null)),
            ("get item 11111111-1111-4111-8111-111111111111", Ok(item())),
        ]);
        steps
    }
    #[test]
    fn create_and_bind_only_after_verified_save() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let vault = Fake::new(create_steps());
        assert_eq!(
            enroll(&vault, &c, "test", &j, || Ok(Some(
                "secret-test-value".into()
            )))
            .unwrap(),
            Some(format!("bw:{ITEM}"))
        );
        assert!(std::fs::read_to_string(&c).unwrap().contains(ITEM));
        assert!(!j.exists());
        assert!(
            !std::fs::read_to_string(&c)
                .unwrap()
                .contains("secret-test-value")
        );
        let bodies = vault.bodies.borrow();
        assert_eq!(bodies[0]["folderId"], FOLDER);
        assert_eq!(bodies[0]["login"]["username"], "root");
    }
    #[test]
    fn cancel_does_not_create_or_edit() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let before = std::fs::read(&c).unwrap();
        let vault = Fake::new(initial(json!([])));
        assert!(
            enroll(&vault, &c, "test", &j, || Ok(None))
                .unwrap()
                .is_none()
        );
        assert_eq!(before, std::fs::read(c).unwrap());
        assert!(!j.exists());
        assert!(vault.bodies.borrow().is_empty());
    }
    #[test]
    fn same_name_without_identity_is_not_overwritten() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let vault = Fake::new(initial(json!([{"name":"PPL / test"}])));
        assert!(enroll(&vault, &c, "test", &j, || panic!("must not prompt")).is_err());
        assert!(vault.bodies.borrow().is_empty());
    }
    #[test]
    fn unknown_create_outcome_is_not_replayed() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let before = std::fs::read(&c).unwrap();
        let mut steps = initial(json!([]));
        steps.extend([
            ("list folders", Ok(json!([{"id":FOLDER,"name":"ssh/PPL"}]))),
            ("create item", Err(anyhow::anyhow!("network disconnected"))),
        ]);
        assert!(
            enroll(&Fake::new(steps), &c, "test", &j, || Ok(Some(
                "secret-test-value".into()
            )))
            .is_err()
        );
        assert!(
            !std::fs::read_to_string(&j)
                .unwrap()
                .contains("secret-test-value")
        );
        let retry = Fake::new(initial(json!([])));
        assert!(enroll(&retry, &c, "test", &j, || panic!("no replay")).is_err());
        assert_eq!(before, std::fs::read(&c).unwrap());
        let mut recovered = initial(json!([item()]));
        recovered.extend([
            ("sync", Ok(Value::Null)),
            ("get item 11111111-1111-4111-8111-111111111111", Ok(item())),
        ]);
        assert!(
            enroll(&Fake::new(recovered), &c, "test", &j, || panic!(
                "already stored"
            ))
            .unwrap()
            .is_some()
        );
    }
    #[test]
    fn external_edit_is_preserved_after_remote_save() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let vault = Fake::new(create_steps());
        let edited = "Host test\n HostName changed.test\n User root\n # purple:askpass bw:\n";
        assert!(
            enroll(&vault, &c, "test", &j, || {
                std::fs::write(&c, edited).unwrap();
                Ok(Some("secret-test-value".into()))
            })
            .is_err()
        );
        assert_eq!(std::fs::read_to_string(c).unwrap(), edited);
        assert!(j.exists());
    }
    #[test]
    fn locked_vault_never_asks_for_server_password() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let vault = Fake::new(vec![("status", Ok(json!({"status":"locked"})))]);
        assert!(enroll(&vault, &c, "test", &j, || panic!("locked")).is_err());
        assert!(vault.bodies.borrow().is_empty());
    }
    #[test]
    fn sync_failure_keeps_config_unbound() {
        let _guard = crate::demo_flag::GLOBAL_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::demo_flag::disable();
        let (_d, c, j) = setup();
        let before = std::fs::read(&c).unwrap();
        let mut steps = create_steps();
        steps.truncate(5);
        steps.push(("sync", Err(anyhow::anyhow!("offline"))));
        assert!(
            enroll(&Fake::new(steps), &c, "test", &j, || Ok(Some(
                "secret-test-value".into()
            )))
            .is_err()
        );
        assert_eq!(before, std::fs::read(c).unwrap());
        assert!(j.exists());
    }
    #[test]
    fn invalid_item_id_never_becomes_config() {
        assert!(id(&json!({"id":"x\nHost evil"})).is_err());
    }
}
