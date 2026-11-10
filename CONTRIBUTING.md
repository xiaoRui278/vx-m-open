# 贡献指南 / Contributing Guide

感谢你对 vx-m-open 的兴趣！无论是报 Bug、提建议、改进文档还是提交代码，都非常欢迎。

Thanks for your interest in contributing! Bug reports, feature requests, docs improvements and code are all welcome.

---

## 🚫 贡献红线（请先读这一节）

为保持本项目的定位与法律边界，以下类型的改动**一律不予合并**：

- 自动化操作、群控、批量消息、批量营销相关
- 防撤回、消息修改、消息伪造相关
- 任何用于绕过第三方应用**风控或身份验证**的能力
- 任何形式的数据上报 / 遥测 / 云同步用户数据（除非有独立的隐私设计文档并经充分讨论）

项目定位是**桌面应用多开与账号隔离管理工具**，不是运营辅助工具。守住这条线，项目才能长期存在。

---

## 🛠 开发环境

| 依赖 | 版本要求 |
| --- | --- |
| Node.js | 20+ |
| pnpm | 9+（仓库使用 `pnpm-lock.yaml`） |
| Rust | stable（`rustup` 默认即可） |
| Tauri 2 前置依赖 | 见 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/) |
| 操作系统 | macOS 11+ 或 Windows 10/11（平台相关代码需在对应系统上运行验证） |

## 🚀 本地运行

```bash
git clone https://github.com/xiaoRui278/vx-m-open.git
cd vx-m-open
pnpm install

pnpm tauri dev      # 开发模式，前端热重载
pnpm tauri build    # 打包 dmg / msi
```

> ⚠️ 平台相关功能（macOS 副本模型 / Windows 互斥体模型）**必须在真实目标系统上手动验证**，CI 只负责构建，不覆盖运行时行为。

## 📐 代码规范

### 界面改动必读 `design.md`

所有 UI 改动**必须先读 [`design.md`](design.md)**。它是本项目锁定的设计系统：

- 需要超出设计系统的样式 → **先修订 `design.md`，再改代码**，禁止页面级私自覆盖
- 色值只能出现在主题令牌块内，组件一律 `var()` 引用；组件里出现一次性 `hex`/`rgba` = 违规
- 状态必须「色 + 形状」双通道表达，禁止仅靠颜色
- 尊重「静默成功」原则：肉眼可见结果的操作不弹 Toast

### Rust 后端

- 所有 Tauri command 保持 `async`，阻塞操作走 `spawn_blocking`
- 策略枚举只表达「引擎内置能力词」，**profile 不得携带可执行内容**（见 `profiles/strategy.rs` 模块注释）
- 新增平台能力时，优先扩展 `platform/` 下的策略实现，而不是在 `commands.rs` 里写分支逻辑

### 前端

- Vue 3 + Pinia + TypeScript，类型定义集中在 `src/types.ts`
- 国际化：所有用户可见文案走 `src/i18n.ts`，**中英双语同步补齐**，禁止硬编码
- 深浅色主题同步适配

### 提交信息

采用 Conventional Commits 风格（仓库现有提交即此风格）：

```
feat: 支持 Windows 微信 4.x 自动识别安装路径
fix: 修复 macOS 副本重建后显示名丢失
docs: 更新微信实测版本至 4.2.0.12
chore: 补充 MIT LICENSE
```

## 🔄 Pull Request 流程

1. Fork 仓库，从 `main` 切出分支（命名建议：`feat/xxx`、`fix/xxx`、`docs/xxx`）
2. 保持 PR **单一主题、尽量小**；大型改动请先开 Issue 讨论方案
3. 填写 PR 描述：**改了什么、为什么、如何验证**
4. 涉及 UI：附改动前后截图（亮色 / 暗色）
5. 涉及平台行为：注明测试的 OS 版本与微信版本
6. 等待 review，可能需要调整

### 我们会关注什么

- 是否破坏 `design.md` 的设计约束
- 是否引入新的外部依赖（能不引就不引）
- 是否扩大了应用的系统权限（capabilities / CSP）
- 是否触及上面的「贡献红线」
- 中英文案是否同步

## 🐛 报告 Bug

开 Issue 时请提供：

- **环境**：OS 版本、微信版本（如 4.2.0.12）、vx-m-open 版本
- **预期行为 vs 实际行为**
- **复现步骤**（越具体越好）
- 相关日志 / 截图

> ⚠️ **安全漏洞请勿公开提交 Issue**，请按 [SECURITY.md](SECURITY.md) 私密上报。

## 💡 功能建议

请先开 Issue 讨论，说明**使用场景**而不只是功能本身。优先级判断标准：
1. 是否服务于「多账号隔离管理」这一核心定位
2. 是否影响大量用户
3. 维护成本是否可控

## 📦 发版流程（维护者）

- 普通提交**不触发**构建
- 发新版：修改版本号 → `git tag vX.Y.Z && git push origin vX.Y.Z`
- CI 自动构建双平台产物并创建正式 Release

## 📜 许可

参与贡献即表示你同意你的贡献以 [MIT License](LICENSE) 授权。
