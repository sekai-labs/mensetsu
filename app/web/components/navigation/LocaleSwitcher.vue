<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Languages, ChevronDown } from 'lucide-vue-next'

const { locale, locales, setLocale } = useI18n()
const isOpen = ref(false)

const availableLocales = [
  { code: 'en', label: 'EN', fullName: 'English' },
  { code: 'ja', label: '日本語', fullName: '日本語' },
  { code: 'fr', label: 'FR', fullName: 'Français' },
]

const currentLocaleDisplay = computed(() => {
  const found = availableLocales.find((l) => l.code === locale.value)
  return found?.label || locale.value.toUpperCase()
})

const changeLocale = async (code: string) => {
  await setLocale(code)
  isOpen.value = false
}
</script>

<template>
  <div class="relative inline-block text-left">
    <button
      type="button"
      @click="isOpen = !isOpen"
      class="inline-flex items-center gap-1.5 px-2 py-1 text-xs font-mono font-medium rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors shadow-2xs"
      :aria-expanded="isOpen"
      aria-label="Switch Language"
    >
      <Languages class="w-3.5 h-3.5 text-slate-400" />
      <span>{{ currentLocaleDisplay }}</span>
      <ChevronDown class="w-3 h-3 text-slate-400" />
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 mt-1.5 w-32 rounded-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-lg py-1 z-50 text-xs font-mono animate-in fade-in zoom-in-95 duration-100"
    >
      <button
        v-for="l in availableLocales"
        :key="l.code"
        type="button"
        @click="changeLocale(l.code)"
        class="w-full text-left px-3 py-1.5 transition-colors flex items-center justify-between"
        :class="locale === l.code ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-bold' : 'text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'"
      >
        <span>{{ l.fullName }}</span>
        <span v-if="locale === l.code" class="text-indigo-600 dark:text-indigo-400 text-[10px]">✓</span>
      </button>
    </div>
  </div>
</template>
