import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

const site = process.env.PUBLIC_SITE_URL || 'http://localhost:3002';

export default defineConfig({
  site,
  output: 'static',
  trailingSlash: 'always',
  compressHTML: true,
  server: {
    host: '0.0.0.0',
    port: 3000,
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
