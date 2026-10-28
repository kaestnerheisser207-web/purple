# Purple 简体中文语言包

基于上游 v3.31.1 的个人 fork。同一可执行文件支持英文与简体中文，不改变 SSH 主机、密钥、命令或钥匙串协议。

- `ppl`：按已保存语言打开。
- `ppl --lang zh-CN` / `ppl --lang en`：本次使用中文 / 英文。
- 主机列表按大写 `L`：立即切换语言并保存。
- `ppl --set-language zh-CN` / `ppl --set-language en`：设置默认语言。

语言包真源为 `locales/zh-CN.json`。运行时优先读取 Purple 配置目录下的 `locales/zh-CN.json`，没有外部包时使用随程序内置的中文包；修改外部语言包后重新打开即可生效，不需要编译。默认位置是 `~/.purple/locales/zh-CN.json`，遵循已有 PURPLE_CONFIG_DIR / XDG_CONFIG_HOME 设置。

覆盖主导航、主机/隧道/命令片段表单、操作按钮、快捷键帮助及常见提示。未收录的上游诊断、云服务商专用说明等保留英文。主机别名、命令内容、服务端输出和 SSH 配置关键词不翻译。语言选择保存在配置目录的 `language` 文件中。

本机安装使用独立可执行文件与 `ppl` 启动器，Homebrew 的官方 `purple` 保留。不要通过个人版本的 `update` 命令更新，否则上游发行包会覆盖中文能力；升级本 fork 后重新编译安装。

验证：`cargo test --lib`、`cargo test --test language_pack`、`cargo build --release --locked`。中文界面用 `ppl --lang zh-CN --demo` 可在演示数据上检查，不连接真实主机。
