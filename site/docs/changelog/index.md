# 更新日志

本站记录 nomic 的已发布版本；完整提交历史见 [GitHub](https://github.com/asionbo/nomic/commits/main)。

## [v0.1.2](https://github.com/asionbo/nomic/releases/tag/v0.1.2) · 2026-11-05

- macOS 事件循环显式使用 `Accessory` 激活策略：托盘常驻菜单栏，不再出现在 Dock 中。

## [v0.1.1](https://github.com/asionbo/nomic/releases/tag/v0.1.1) · 2026-11-05

- 新增中文 README 与中英文界面切换（托盘菜单或 `--lang zh`），默认仍为英文。
- 托盘图标改为高分辨率绘制，并加入 macOS 应用图标生成脚本。
- 发布流程加入签名、公证与多平台打包，同时完善 README、`.gitignore` 等文档。
- 修复若干界面与注入逻辑的小问题。

## [v0.1.0](https://github.com/asionbo/nomic/releases/tag/v0.1.0) · 2026-11-05

- 首个公开版本：手机扫码 → 局域网 WebSocket → 剪贴板 → 模拟 ⌘/Ctrl+V 的完整链路，macOS 已验证。
- 加入每次启动重新生成的令牌、Host / Origin 校验、连接数与消息大小限制。
- 内嵌二维码 SVG 与手机输入页，内置中英文界面文案，页面使用严格 CSP。
- 加入 GitHub Actions 的 lint 与 release 工作流，并补充 GPL-3.0 许可证。

::: info 关于未发布改动
`v0.1.2` 之后的未发布改动：托盘菜单新增「粘贴后自动回车」开关（默认关闭），开启后粘贴成功会追加模拟回车键；另有 lint 修复。它们会出现在下一个版本中。
:::
