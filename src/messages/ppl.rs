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
