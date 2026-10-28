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

## Bitwarden 首次连接自动保存

新增/编辑主机，在“密码来源”选择 Bitwarden 并保留 `bw:`（不填名称或 UUID）。第一次从主机列表 Enter 连接，或运行 `ppl 主机别名` / `ppl --connect 主机别名` 时：

1. 如有需要，先输入 Bitwarden 主密码解锁 CLI（首次仍需 `bw login`）。
2. 输入该服务器的登录密码，Esc 可取消。
3. 自动在个人密码库的 `ssh/PPL` 文件夹创建 `PPL / 主机别名`，保存用户名、密码及服务器地址。
4. 同步并确认条目存在后，将 `bw:UUID` 回填 SSH 配置，再开始 SSH 登录。

这与钥匙串流程一样在 SSH 登录前保存密码，不宣称密码已通过服务器认证。登录失败可以在 Bitwarden 中修正密码；已绑定条目不会被此流程覆盖。钥匙串切换到 Bitwarden 时保留原钥匙串记录。

若有同名但无法确认归属的条目，停止并要求核对；创建过程中网络中断、返回无效或配置被外部编辑，会保留仅包含身份/ID 的恢复日志，不保存服务器密码。重试先同步和查找带对应标记的条目，结果不明确时不会重复创建。后台文件浏览/容器/隧道不弹出建档输入，请先完成一次主机连接。
