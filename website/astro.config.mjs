import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import { remarkZagrosDocs } from './remark-zagros-docs.mjs';
import { rehypeZagrosLinks } from './rehype-zagros-links.mjs';

// GitHub Pages custom domain serves the site at the root.
export default defineConfig({
  site: 'https://zagros.mystronics.com',
  base: '/',
  output: 'static',
  integrations: [sitemap()],
  markdown: { remarkPlugins: [remarkZagrosDocs], rehypePlugins: [rehypeZagrosLinks] },
  build: { format: 'directory' },
});
