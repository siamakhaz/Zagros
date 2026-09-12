import { defineConfig } from 'astro/config';

// GitHub Pages: https://siamakhaz.github.io/Zagros/
// For local dev the base is ignored; for build it prefixes all assets.
export default defineConfig({
  site: 'https://siamakhaz.github.io',
  base: '/Zagros/',
  output: 'static',
  build: { format: 'directory' },
});
