# 更新日志 / Changelog

本文件记录 vx-m-open 的所有重要变更。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [未发布 / Unreleased]

### Added
- 补充 `LICENSE`（MIT），`package.json` / `Cargo.toml` 增加 license 与 repository 字段
- 新增 `SECURITY.md` 安全政策与 `CONTRIBUTING.md` 贡献指南

---

## [1.2.0] - 2026-11-04

- 🏗️ 多开引擎抽象重构：macOS 副本模型 / Windows 互斥体模型统一到引擎层，为后续扩展铺路
- 🔄 「全部重建」事务化：逐目标执行、单项失败不再中断整批，其余副本照常重建
- 🔒 移除匿名安装统计，下载量以 GitHub Release 计数为准，保持零遥测
- 📖 说明微信 4.1.7+ 同一设备同时登录上限 4 个账号为官方限制

---

## [1.1.0] - 2026-10-26

### Added
- **应用内自动更新**：接入 Tauri updater 插件，更新包经 minisign 签名校验
- **一键更新**：更新横幅可直接下载安装并重启，失败自动降级为官网下载
- **双通道更新源**：官网 `vxmo.cting.vip` 优先，GitHub Release 兜底
- **Windows 安装器双语**：NSIS 跟随系统语言（中/英），MSI 提供 zh-CN / en-US 两个语言包

### Changed
- 更新检测改为优先读取官网发布信息，国内网络下更快更稳，不受 GitHub API 限流影响
- CI 发布流程自动注入更新签名私钥并生成 `latest.json`

### Removed
- **取消匿名安装统计**：曾短暂加入的匿名统计已完全移除，下载量以 GitHub Release 计数为准，项目保持**零遥测**承诺

---

## [1.0.2] - 2026-10-22

### Added
- Windows 实例支持临时备注（跟随进程生命周期）；启动新实例时可先填备注再启动
- 设置页重组为「通用 / 环境 / 关于 / 支持作者」四组，版本与检查更新合并为一行
- 「支持作者」捐赠入口（微信 / 支付宝收款码）

### Fixed
- **安全性**：设置 CSP 白名单；`aliases.json` 改为原子写入；rename 失败时清理临时文件
- **进程安全**：收割残留子进程前校验 exe 名，防止 PID 复用误杀无关进程
- Windows 关闭实例时清理残留帮手子进程
- 运行实例检测改用进程映像名匹配，修复漏检问题
- 进程刷新显式指定 exe/cmd 字段，修复 Windows 子进程过滤失效
- 更新检查增加 5s 超时；超时覆盖响应体读取，异常响应不再误判为「已是最新」
- 互斥体候选名放宽以覆盖微信 4.x，启动超时按进程存活判定成败并保留失败原因
- 弹框点击空白不再关闭，改为 Esc 关闭防误触

### Performance
- `sysinfo` 改为最小化进程刷新，降低 3 秒轮询开销
- 窗口不可见时暂停轮询

---

## [1.0.1] - 2026-10-22

### Added
- 设置页新增「官网」入口，直达 vxmo.cting.vip 下载最新安装包
- 侧边栏底部新增 GitHub 入口按钮

### Changed
- 版本检查改为优先读取官网发布信息，国内网络下响应更快

---

## [1.0.0] - 2026-10-21

首个正式版本。

### Added
- **macOS 副本模型**：APFS 克隆 + 改 Bundle ID + ad-hoc 重签名，约 10 秒生成独立副本；自定义名称同步显示在 Finder / 启动台 / Dock
- **Windows 本体多开**：挂起启动 + 关闭单实例互斥体，不产生副本文件，支持微信 3.x（`WeChat.exe`）与 4.x（`Weixin.exe`）
- **环境检测**：macOS 扫描应用目录、Windows 读注册表，自动识别微信安装路径与版本
- **实例面板**：实时显示运行状态、PID、启动时间；启动 / 激活 / 关闭一处完成
- **版本同步**：微信升级后提示重建副本，账号数据保留、无需重新登录
- **中英双语界面**，跟随系统语言，可手动切换
- **明暗主题**：亮色 / 暗色 / 跟随系统
- **锁定设计系统** `design.md`：平台母语式界面语言（macOS 系统设置风格 / Windows Fluent 风格）

---

## 版本兼容说明

| 微信版本 | macOS | Windows |
| --- | --- | --- |
| macOS 微信 4.x | ✅ 已在 4.2.0.12 实测 | — |
| Windows 微信 3.x（`WeChat.exe`） | — | ✅ 支持 |
| Windows 微信 4.x（`Weixin.exe`） | — | ✅ 支持 |

> ⚠️ 微信 4.1.7+ 起，同一设备同时登录上限为 **4 个账号**，第 5 个会提示「暂无法在当前设备登录更多账号」。此为**微信官方限制**，非本工具问题。
>
> ⚠️ 微信版本更新可能使多开机制失效，本工具会尽力跟进适配，但不作任何保证。详见 [DISCLAIMER.md](DISCLAIMER.md)。

[未发布 / Unreleased]: https://github.com/xiaoRui278/vx-m-open/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/xiaoRui278/vx-m-open/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/xiaoRui278/vx-m-open/compare/v1.0.2...v1.1.0
[1.0.2]: https://github.com/xiaoRui278/vx-m-open/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/xiaoRui278/vx-m-open/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/xiaoRui278/vx-m-open/releases/tag/v1.0.0
