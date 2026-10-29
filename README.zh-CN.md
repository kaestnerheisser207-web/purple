# Purple 简体中文语言包

基于上游 v3.31.1 的个人 fork。同一可执行文件支持英文与简体中文，不改变 SSH 主机、密钥、命令或钥匙串协议。

- `ppl`：按已保存语言打开。
- `ppl --lang zh-CN` / `ppl --lang en`：本次使用中文 / 英文。
- 主机列表按大写 `L`：立即切换语言并保存。
- `ppl --set-language zh-CN` / `ppl --set-language en`：设置默认语言。

语言包真源为 `locales/zh-CN.json`。运行时优先读取 Purple 配置目录下的 `locales/zh-CN.json`，没有外部包时使用随程序内置的中文包；修改外部语言包后重新打开即可生效，不需要编译。默认位置是 `~/.purple/locales/zh-CN.json`，遵循已有 PURPLE_CONFIG_DIR / XDG_CONFIG_HOME 设置。

覆盖主导航、主机/隧道/命令片段表单、操作按钮、快捷键帮助及常见提示。未收录的上游诊断、云服务商专用说明等保留英文。主机别名、命令内容、服务端输出和 SSH 配置关键词不翻译。语言选择保存在配置目录的 `language` 文件中。

本机安装使用独立可执行文件与 `ppl` 启动器，Homebrew 的官方 `purple` 保留。从 PPL 3.31.2 起，`ppl update` 只读取 `kaestnerheisser207-web/purple` 的 `ppl-v…` Release，校验 SHA256 后替换当前独立二进制，不会下载上游发行包。3.31.1 的旧程序仍指向上游，首次迁移需从本 fork 的 GitHub Actions Release 安装一次，不能用旧版自更新。

验证：`cargo test --lib`、`cargo test --test language_pack`、`cargo build --release --locked`。中文界面用 `ppl --lang zh-CN --demo` 可在演示数据上检查，不连接真实主机。

## Bitwarden 首次连接自动保存

新增/编辑主机，在“密码来源”选择 Bitwarden 并保留 `bw:`（不填名称或 UUID）。第一次从主机列表 Enter 连接，或运行 `ppl 主机别名` / `ppl --connect 主机别名` 时：

1. 如有需要，先输入 Bitwarden 主密码解锁 CLI（首次仍需 `bw login`）。
2. 输入该服务器的登录密码，Esc 可取消。
3. 自动在个人密码库的 `ssh/PPL` 文件夹创建 `PPL / 主机别名`，保存用户名、密码及服务器地址。
4. 服务端确认创建、本地回读校验成功后，将 `bw:UUID` 回填 SSH 配置，再开始 SSH 登录。建档前保留一次同步与查重，已确认创建后不再重复全库同步。

这与钥匙串流程一样在 SSH 登录前保存密码，不宣称密码已通过服务器认证。登录失败可以在 Bitwarden 中修正密码；已绑定条目不会被此流程覆盖。钥匙串切换到 Bitwarden 时保留原钥匙串记录。

若有同名但无法确认归属的条目，停止并要求核对；创建过程中网络中断、返回无效或配置被外部编辑，会保留仅包含身份/ID 的恢复日志，不保存服务器密码。重试先同步和查找带对应标记的条目，结果不明确时不会重复创建。后台文件浏览/容器/隧道不弹出建档输入，请先完成一次主机连接。

## 改名与连接提示

在 ppl 中修改已绑定主机的名称，会同步它自动创建的 Bitwarden 条目名称和归属标记；UUID、密码、TOTP 和其他字段保持不变。保存后需要解锁时在本机终端输入主密码。网络失败会显示“名称尚未同步”，仅含元数据的待处理记录保存在状态目录 `bitwarden-renames/`，下次连接重试；连续改名会合并为最新名称。没有 ppl 归属标记的外部条目不会自动改名，Bitwarden 内手工更改的名称也不会被覆盖。只改 Bitwarden 名称不影响 UUID 取密码。

首次连接新主机时显示完整 SSH 指纹，由用户明确确认。后台任务和指纹变化不会自动接受。已经进入服务器的普通 shell 退出，不再因最后一条命令返回非零而报告“SSH 连接失败”；连接错误摘要不会把“指纹已保存”提示当成失败原因。

## Fork 更新与 Actions 发布

- `upstream-release` 是纯上游正式版分支。GitHub Actions 每天北京时间 09:17 查询上游最新稳定 Release，并快进同步；不跟随未发布的 master，不自动合入定制分支，不强制覆盖分叉历史。GitHub 定时任务可能延迟，亦可手动运行 Sync upstream release。
- `feature/zh-cn` 是默认分支及整合发布分支。先在开发分支合并 `origin/upstream-release`，审阅新功能、补齐中文并回归 Bitwarden、SSH 和更新源；完成审核后再合入并推送 `feature/zh-cn`。
- 每次推送自动运行 Publish PPL integration。工作流选择高于已有 `ppl-v…` 标签的版本，在仅供发布的提交中更新 Cargo.toml、Cargo.lock 和 CHANGELOG.md；不会向整合分支回写机器人提交。发布提交中的 `PPL-Source` 记录对应源码提交，同一提交重跑复用原标签。
- 测试、格式检查、Clippy、SSH/Bitwarden 冒烟测试通过后，构建六个平台，全部成功才发布个人 Release 和 SHA256 文件。失败时没有新 Release；修复后推送，或重跑原 Actions。
- 手动推送 `ppl-v…` 标签和手动运行 Release 工作流仍可使用。不要移动已经发布的标签。PPL 版本号独立递增，不代表已整合相同版本的上游；当前整合源码仍基于上游 v3.31.1，v3.32.0 留在同步分支待审核。
- Homebrew Tap、AUR、上游网站、营销素材和云 API 监控仅允许原上游仓库运行，避免向个人发布分支写入未经审核的修改。
- 首次从个人 Release 下载匹配平台的 `purple-<版本>-<平台>.tar.gz` 和 `.sha256`，核验后安装到 `~/.local/lib/purple-zh/purple`。之后打开 ppl 会检查个人 Release（缓存一小时），显示更新提示，执行 `ppl update` 安装；没有系统推送或静默安装。

发行包只使用 GitHub Actions 产物。外部 `~/.purple/locales/zh-CN.json` 会优先于内置语言包；需要随发行包更新翻译时，应备份后移走旧外部包，使用程序内置包。

## 评审修复与恢复边界

旧式 `bw:条目名称` 继续按名称取密码，不参与托管改名。绑定 UUID 的主机修改别名、地址、用户名或端口时都会排队同步归属标记；仅处理个人密码库条目，组织条目不自动编辑。新建条目回读时核对输入密码，确认一致后才回填 UUID。

创建中断或主机身份改变时，错误会显示 `bitwarden-enrollment/` 中的具体 journal 路径。先执行 `bw sync` 并检查个人密码库和回收站：已有目标条目则绑定其 UUID；只有确认没有创建条目且没有注册过程运行时，才把这一份 journal 改名为 `.reviewed-backup` 后重试。不要为重置流程删除密码库条目。改名同步的错误同样给出 `bitwarden-renames/` 中的记录路径；确认是有意的手动改名后，可在关闭 ppl 时把该记录改名备份，停止重试，UUID 和密码保持不变。

坏的外部语言包会明确警告并回退到内置包，不阻断 CLI；未覆盖的词条继续使用内置翻译。demo 或 `--lang` 临时语言下按 L 只改变本次会话，常规模式会保存偏好。MCP 错误和审计日志保持英文，中文详情标签按显示列宽对齐。
