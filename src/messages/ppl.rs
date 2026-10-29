//! Personal fork language and vault status messages.
pub fn language_error(error: &impl std::fmt::Display) -> String {
    format!("Language: {error}")
}
pub fn language_selected(language: &str) -> String {
    format!("Language / 语言: {language}")
}
pub fn rename_pending(error: &impl std::fmt::Display) -> String {
    format!("Bitwarden 名称尚未同步，下次连接会重试：{error}")
}
pub fn host_renamed_pending(error: &impl std::fmt::Display) -> String {
    format!("主机已改名；{}", rename_pending(error))
}
pub const PASSWORD_BOUND: &str = "Bitwarden 密码已保存并绑定，正在连接…";

/// Recovery is explicit because a network failure cannot prove creation failed.
pub fn enrollment_recovery(journal: &std::path::Path) -> String {
    format!(
        "Recovery journal (if present): {}. Run bw sync and inspect the personal vault and trash. If the intended item exists, bind its UUID. Only after confirming no item was created and no enrollment is running, move this journal to a .reviewed-backup filename and retry. Never remove vault items as part of this reset.",
        journal.display()
    )
}
pub fn rename_recovery(journal: &std::path::Path) -> String {
    format!(
        "Pending metadata: {}. Verify the host UUID and vault ownership/name. To stop retrying after a deliberate manual change, move this file to a .reviewed-backup filename while Purple is closed; credentials and the UUID binding remain unchanged.",
        journal.display()
    )
}
pub const HOST_SYNC_MISSING: &str = "Saved host is missing; Bitwarden metadata was not changed";
