# 免责声明 / Disclaimer

**最后更新：2026 年 11 月**

请在使用 vx-m-open（以下简称「本软件」）之前仔细阅读本声明。下载、安装或使用本软件，即表示你已阅读、理解并同意接受以下全部条款。若不同意，请勿使用本软件。

---

## 1. 与腾讯 / 微信无关

- 本软件是一个**独立的开源工具**，由个人开发者维护，与腾讯控股有限公司、微信团队**没有任何关联、授权、合作或背书关系**。
- 本软件不属于微信官方产品，也**未获得微信官方任何形式的认可或支持**。
- 「微信」、「WeChat」及相关商标、标识的所有权归腾讯公司所有，本声明中提及仅用于指代目标应用，不构成商标使用授权。

## 2. 功能性质与技术边界

本软件的作用是帮助用户在同一台电脑上运行多个应用实例，以便分别登录不同账号。具体而言：

- **macOS**：克隆应用并修改副本的 Bundle ID 与签名，使副本落入独立容器
- **Windows**：在应用启动阶段关闭其单实例互斥体，使应用允许多个进程共存

本软件**不修改目标应用本体的任何文件**，也**不读取、存储、上传或备份**任何聊天记录、账号凭据或其他用户数据。

## 3. 使用风险自负

**你理解并同意，使用本软件的风险由你本人完全承担。** 特别包括但不限于：

- **账号风险**：多开行为可能触发目标应用的风控机制，可能导致账号被限制登录、功能受限、封号或其他处理。本软件**不保证、也无意规避**任何风控或安全检测。
- **兼容性风险**：目标应用的版本更新可能使本软件的多开机制失效、行为异常或导致副本数据不可用。开发者会尽力跟进适配，但**不承诺适配时效或适配成功**。
- **数据风险**：副本数据的完整性、可用性由使用者自行负责。在进行重建、删除等操作前，建议自行备份重要数据。
- **系统风险**：本软件涉及挂起进程、关闭系统对象、重签名应用等系统级操作，理论上存在影响系统稳定性的可能。

对于上述任何直接或间接损失，**开发者在法律允许的最大范围内不承担责任**。

## 4. 无担保

本软件按「**现状（AS IS）**」和「**现有（AS AVAILABLE）**」基础提供，不附带任何形式的明示或默示担保，包括但不限于适销性、特定用途适用性、非侵权性、安全性、可靠性或持续可用性的担保。使用本软件产生的全部结果由使用者自行判断与承担。

## 5. 使用规范

使用者承诺**合法、正当**地使用本软件，并同意：

- 不将本软件用于垃圾营销、群控、欺诈、骚扰、侵犯他人隐私或其他违法违规活动
- 不将本软件用于规避任何应用的安全机制、身份验证或访问控制
- 遵守所在地区适用的法律法规以及目标应用的服务条款

开发者**明确不支持、不鼓励**上述任何违规用途；一旦发现，开发者有权拒绝提供任何形式的支持。

## 6. 开源许可与责任限制

- 本软件以 [MIT License](LICENSE) 开源发布。MIT 协议中的「无担保」与「责任限制」条款构成本声明的一部分，并以其中规定为准。
- 在任何情况下，无论基于合同、侵权（含过失）或其他法律理论，开发者均**不对任何间接、附带、特殊、惩罚性损害，或任何利润损失、数据丢失、业务中断**承担责任，即使已被告知此类损害的可能性。
- 部分司法辖区不允许排除或限制默示担保或责任限制，上述限制可能对你不适用，此时以适用法律允许的最大范围为准。

## 7. 免责声明的变更

开发者可能不定期修订本声明，修订后的版本自公布之日起生效。继续使用本软件即视为接受修订后的条款。重大变更会在 [CHANGELOG.md](CHANGELOG.md) 中说明。

---

## English Summary

This software is an **independent open-source tool** and is **not affiliated with, endorsed by, or supported by Tencent or the WeChat team**. All trademarks belong to their respective owners.

Use of this software is **entirely at your own risk**. It is provided **"AS IS" without warranty of any kind**. The developer makes **no guarantee** that multi-instance functionality will continue to work after target-app updates, and **does not guarantee** that your account will not be restricted or banned by the target application's risk-control systems.

This software does **not** modify the target application's own files, and does **not** read, store, or upload any chat records, credentials, or user data.

The developer shall **not be held liable** for any direct or indirect damages, including but not limited to data loss, account suspension, or loss of profits, arising from the use of this software.

See [LICENSE](LICENSE) (MIT) for the full license terms.
