export default {
  darkMode: 'class',
  content: [
    './components/**/*.{vue,js,ts}',
    './layouts/**/*.vue',
    './pages/**/*.vue',
    './features/**/*.{vue,js,ts}',
    './plugins/**/*.{js,ts}',
    './app.vue',
    './error.vue',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#4f46e5',
          hover: '#4338ca',
        },
        surface: {
          base: 'var(--surface-card, #ffffff)',
          elevated: 'var(--bg-main, #f8fafc)',
          border: 'var(--border-color, #e2e8f0)',
        },
        text: {
          primary: 'var(--text-primary, #0f172a)',
          muted: 'var(--text-muted, #64748b)',
        },
        slate: {
          850: '#151f32',
          950: '#0b0f19',
        },
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'monospace'],
      },
    },
  },
  plugins: [],
}
