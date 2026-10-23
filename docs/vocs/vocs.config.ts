import { defineConfig } from 'vocs/config'
import { sidebar } from './sidebar'
import { basePath } from './redirects.config'

export default defineConfig({
  title: 'AlteredCarbon (Reth port)',
  description: 'Personal opBNB / AlteredCarbon experiment ported onto a Reth version — not upstream Reth.',
  accentColor: 'light-dark(#1f1f1f, #ffffff)',
  srcDir: 'docs',
  logoUrl: '/logo.png',
  iconUrl: '/logo.png',
  ogImageUrl: '/reth-prod.png',
  outDir: 'docs/dist',
  renderStrategy: 'full-static',
  sidebar,
  basePath,
  search: {
    fuzzy: true
  },
  topNav: [
    { text: 'Run', link: '/run/ethereum' },
    { text: 'SDK', link: '/sdk' },
    { text: 'Rustdocs', link: '/docs' },
    { text: 'GitHub', link: 'https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon' },
    {
      text: 'v2.8.0',
      items: [
        {
          text: 'Releases',
          link: 'https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/releases'
        },
      ]
    }
  ],
  socials: [
    {
      icon: 'github',
      link: 'https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon',
    },
  ],
  editLink: {
    link: "https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/edit/main/docs/vocs/docs/pages/:path",
  },
  vite: {
    plugins: [
      {
        name: 'transform-summary-links',
        apply: 'serve', // only during dev for faster feedback
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      },
      {
        name: 'transform-summary-links-build',
        apply: 'build', // only apply during build
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      }
    ]
  }
})
