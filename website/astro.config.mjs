import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// GitHub Pages custom domain serves the site at the root.
export default defineConfig({
  site: 'https://zagros.mystronics.com',
  base: '/',
  output: 'static',
  integrations: [sitemap()],
  build: { format: 'directory' },
});
