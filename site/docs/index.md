---
layout: home

hero:
  name: nomic
  text: 手机变成电脑的无线键盘
  tagline: 扫码配对，文字与手机语音输入直达电脑光标。纯 Rust 原生实现，单文件约 1.5 MB。
  image:
    src: /hero.svg
    alt: 手机通过局域网把文字发送到电脑
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/
    - theme: alt
      text: 下载最新版
      link: https://github.com/asionbo/nomic/releases/latest
    - theme: alt
      text: 查看更新日志
      link: /changelog/

features:
  - icon: 🎙️
    title: 手机语音听写
    details: 用手机键盘自带的麦克风输入，网页本身不申请麦克风权限。适合电脑没有麦克风、又想语音输入的场景。
  - icon: 📱
    title: 扫码即连
    details: 启动后在电脑浏览器打开二维码页，手机扫一下就能开始输入。无需账号、不经过云端。
  - icon: 🔒
    title: 只在局域网
    details: 每次启动生成新的 128-bit 令牌，所有 HTTP 路由都校验令牌与 Host；轮换令牌会立即断开旧连接。
  - icon: 🪶
    title: 轻量原生
    details: 一个托盘进程，没有 WebView、Electron 或 Tauri，也没有前端框架和图像库；桌面端只是 Rust 二进制。
  - icon: 🌏
    title: 中文与 emoji
    details: 文本通过剪贴板传递并自动模拟 ⌘V / Ctrl+V，不逐字模拟按键，中文和 emoji 都能原样输入。
  - icon: 🖥️
    title: 跨平台
    details: 支持 macOS（通用包）、Windows x86_64 与 Linux x86_64；自动粘贴不可用时自动降级为仅剪贴板。
---

## 它是怎么工作的？

```mermaid
flowchart LR
  A["手机浏览器<br/>打字 / 语音听写"] -->|"WebSocket · 同一 Wi-Fi"| B["电脑上的 nomic<br/>Rust 托盘进程"]
  B --> C["写入系统剪贴板"]
  C --> D["模拟 ⌘V / Ctrl+V<br/>粘贴到光标处"]
```

电脑端启动时会在本机打开一个二维码页面；手机扫码后打开输入页。两台设备需要在同一个 Wi-Fi 下，流量不离开局域网。

::: tip 自动粘贴需要系统权限
macOS 需要在「系统设置 → 隐私与安全性 → 辅助功能」里允许 nomic；Windows 首次启动需要允许防火墙。没有权限时文字仍会写入剪贴板，手动按 ⌘/Ctrl+V 即可。
:::
