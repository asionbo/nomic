import { defineConfig } from 'vitepress'

// GitHub Pages 的仓库页会带 /nomic/ 前缀；本地预览与自定义域名则不需要。
// 部署前可用 `VITEPRESS_BASE=/<repo-name>/ npm run build` 覆盖。
const base = process.env.VITEPRESS_BASE || '/'

export default defineConfig({
  base,
  lang: 'zh-CN',
  title: 'nomic',
  description: '把手机当作电脑的无线键盘：扫码配对，文字与手机语音输入直达电脑光标。',
  lastUpdated: true,
  cleanUrls: true,
  head: [
    ['meta', { name: 'theme-color', content: '#2563eb' }],
    ['link', { rel: 'icon', href: `${base}favicon.svg` }]
  ],
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '指南', link: '/guide/' },
      { text: '更新日志', link: '/changelog/' },
      { text: 'GitHub', link: 'https://github.com/asionbo/nomic' }
    ],
    sidebar: [
      { text: '开始使用', link: '/guide/' },
      { text: '界面预览', link: '/screenshots/' },
      { text: '更新日志', link: '/changelog/' }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/asionbo/nomic' }
    ],
    footer: {
      message: '基于 GPL-3.0 许可证发布 · 纯 Rust / 局域网 / 无云端',
      copyright: '© 2026 nomic contributors'
    },
    editLink: {
      pattern: 'https://github.com/asionbo/nomic/edit/main/site/docs/:path',
      text: '在 GitHub 上编辑此页'
    },
    docFooter: { prev: '上一页', next: '下一页' },
    outlineTitle: '本页目录',
    lastUpdatedText: '最后更新'
  }
})
