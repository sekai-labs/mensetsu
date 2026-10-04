<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
  logoText?: string
}>()
</script>

<template>
  <div class="min-h-[calc(100vh-10rem)] flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-md">
      <div class="flex justify-center">
        <div class="w-10 h-10 rounded-lg bg-indigo-600 flex items-center justify-center text-white font-bold text-lg tracking-wider shadow-sm">
          {{ logoText || '面' }}
        </div>
      </div>
      <h2 class="mt-4 text-center text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ title }}
      </h2>
      <p v-if="subtitle" class="mt-1 text-center text-xs text-slate-500 dark:text-slate-400">
        {{ subtitle }}
      </p>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-white dark:bg-slate-900 py-8 px-4 shadow-sm border border-slate-200 dark:border-slate-800 rounded-lg sm:px-10">
        <slot />

        <div v-if="$slots.footer" class="mt-6 border-t border-slate-200 dark:border-slate-800 pt-4 text-center">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </div>
</template>
