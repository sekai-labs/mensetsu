import { fileURLToPath } from 'node:url'
import { defineNuxtConfig } from 'nuxt/config'
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false },
  modules: ['@nuxtjs/tailwindcss'],
  css: ['~/assets/css/main.css'],
  alias: {
    '@mensetsu/contracts': fileURLToPath(new URL('../../contracts/rest/src/index.ts', import.meta.url)),
  },
  vite: {
    optimizeDeps: {
      include: [
        '@vue-flow/core',
        '@vueuse/core',
        'lucide-vue-next',
      ],
    },
  },
  app: {
    head: {
      title: 'Mensetsu | Evidence-Backed Technical Interview Workspace',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        { name: 'description', content: 'AI-driven, evidence-backed technical interview preparation workspace for engineers and systems architects.' }
      ],
      link: [
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap' }
      ]
    }
  }
})
