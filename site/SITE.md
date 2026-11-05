# nomic 展示页

本仓库的静态展示页，使用 [VitePress](https://vitepress.dev/) 构建并部署到 GitHub Pages。内容只有项目说明、使用指南、界面预览和更新日志；Rust 程序本身在仓库根目录的 `src/`。

整个站点都收在 `site/` 里。GitHub Actions 工作流因平台限制仍放在仓库的 `.github/workflows/pages.yml`。

## 本地开发

```sh
cd site
npm install
npm run dev       # http://localhost:5173
npm run build     # 产物在 site/docs/.vitepress/dist/
npm run preview   # 预览构建结果
```

## 部署到 GitHub Pages

首次发布时：

1. 打开仓库 **Settings → Pages → Build and deployment**，把 **Source** 设为 **GitHub Actions**。
2. 把站点改动推送到 `main`，或在 Actions 页手动运行 **Deploy VitePress site to GitHub Pages**。
3. 发布后的地址是 `https://asionbo.github.io/nomic/`。

工作流只对 `site/**` 和自身文件的改动触发，且用 `VITEPRESS_BASE=/nomic/` 构建，所以不会与 Rust 的 lint / release 工作流互相干扰。若改用自己的域名，把 `VITEPRESS_BASE` 设为 `/`，并在 `site/docs/public/` 放一个名为 `CNAME`、内容为域名的文件即可。

## 目录结构

```text
site/
├── .gitignore
├── SITE.md                     # 本文件
├── package.json
├── package-lock.json
├── docs/
│   ├── .vitepress/config.mjs   # 站点标题、导航、base
│   ├── public/
│   │   ├── images/             # 界面预览图（WebP）
│   │   ├── favicon.svg
│   │   └── hero.svg
│   ├── index.md                # 首页
│   ├── guide/index.md          # 快速开始
│   ├── screenshots/index.md    # 界面预览
│   └── changelog/index.md      # 更新日志
└── scripts/make_screenshots.py
../.github/workflows/pages.yml  # Pages 构建与部署
```

## 更新截图

在 `site/` 下运行脚本。它会读取仓库根目录的 `assets/*.html`，在临时目录里填充示例文案，用无头 Chrome 渲染并转成 WebP。二维码是固定随机图案、令牌是示例值，**不是有效配对链接**。

```sh
cd site
python3 scripts/make_screenshots.py
```

需要 macOS 上的 Google Chrome 与 `cwebp`（`brew install webp`）。页面改成英文时，脚本里的中文文案也要同步更新。

## 发布新版本

编辑 `site/docs/changelog/index.md`，提交并推送到 `main` 即可；部署工作流会自动更新 Pages。产品版本仍按原有 `v*` 标签流程发布。
