# 安全政策 / Security Policy

感谢你帮助 vx-m-open 变得更安全。本项目涉及挂起进程、关闭系统对象、重签名应用等敏感操作，我们对安全问题采取**最高优先级**处理。

Thanks for helping keep vx-m-open secure. This project performs sensitive system operations (process suspension, handle closing, code re-signing), so we treat security reports with top priority.

---

## 📌 优先维护范围 / Supported Versions

| 版本 | 是否接收安全更新 |
| --- | --- |
| 最新 Release | ✅ 是 |
| 上一个 Release | ✅ 是（关键漏洞） |
| 更早版本 | ❌ 否，请升级 |

## 📮 漏洞上报 / Reporting a Vulnerability

**请勿公开提交 Issue 或在任何公开渠道讨论未修复的漏洞。**

请选择以下任一方式：

1. **GitHub 私密上报（推荐）**：进入仓库 [Security → Report a vulnerability](../../security/advisories/new) 填写表单

请尽量包含：

- 漏洞类型与影响（如：本地提权、任意代码执行、数据泄露、更新劫持）
- 复现步骤 / PoC / 概念验证
- 影响的操作系统与版本（macOS / Windows）、微信版本、vx-m-open 版本
- 你对风险等级的评估

### 响应承诺 / Response SLA

| 阶段 | 时间 |
| --- | --- |
| 确认收到 | 48 小时内 |
| 初步评估与定级 | 5 个工作日内 |
| 修复方案或缓解措施 | 严重漏洞 7 天内，其余按计划 |
| 公开披露 | 修复发布后，且与上报者协商一致 |

维护者为个人开发者，非工作时间可能延迟，但会保持沟通。

## 🎯 安全范围 / In Scope

我们**特别关注**以下方面的漏洞：

- **更新通道安全**：`plugins.updater` 的签名绕过、`latest.json` 被篡改导致的任意代码执行、中间人攻击
- **权限越界**：应用超出声明的 `capabilities` 权限、CSP 绕过
- **命令注入 / 任意命令执行**：克隆、重签名、启动、终止流程中对路径 / 参数处理不当
- **账号与数据安全**：微信副本数据意外泄露、副本删除后残留敏感数据、备份迁移中的数据外泄
- **提权**：利用本工具在系统上获得超出用户预期的权限

## 🚫 不在范围 / Out of Scope

以下问题属于产品边界或第三方行为，请不要作为安全漏洞上报：

- **微信官方风控 / 账号限制**：包括扫码上限、登录异常提示、账号被限制等。本工具不保证规避任何风控，风险由使用者自行承担
- **微信版本更新导致多开机制失效**：这是兼容性问题，请提交普通 Issue
- **微信自身漏洞**：请向腾讯安全应急响应中心（[TSRC](https://security.tencent.com)）报告
- **社会工程学、物理攻击、DoS 攻击**
- **已在其他渠道公开的漏洞**

## 🔍 我们的操作边界 / Operational Boundaries

为便于你审计，本工具在系统层面**会做**与**不会做**的事：

**会做（均已开源可审计）：**
- macOS：克隆微信 `.app`、修改副本的 Bundle ID、对副本执行 ad-hoc 重签名
- Windows：挂起微信启动进程、关闭其单实例互斥体句柄、恢复执行
- 读取微信安装路径与版本信息（注册表 / 应用目录 / Info.plist）
- 检查更新、下载并安装新版应用

**不会做（未来也不会，除非设计文档先行变更并公示）：**
- 读取、上传、备份任何微信聊天记录或账号数据
- 修改微信本体的任何文件
- 自动化操作、群控、消息收发、防撤回、消息修改
- 任何形式的遥测数据上报

## 🙏 致谢 / Acknowledgments

负责任上报的安全研究者将在修复发布后于本文件中致谢（如你愿意署名）。
