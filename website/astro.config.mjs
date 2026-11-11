import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// GitHub Pages: https://siamakhaz.github.io/Zagros/
// For local dev the base is ignored; for build it prefixes all assets.
export default defineConfig({
  site: 'https://siamakhaz.github.io',
  base: '/Zagros/',
  output: 'static',
  integrations: [sitemap()],
  build: { format: 'directory' },
});
