<script setup lang="ts">
defineProps<{
  currentStep: number
  steps: { id: number; title: string; subtitle?: string }[]
}>()
</script>

<template>
  <nav aria-label="Progress" class="py-4 overflow-x-auto scrollbar-none">
    <ol class="flex items-center min-w-[500px] sm:min-w-0 justify-between border-b border-slate-200 dark:border-slate-800 pb-4">
      <li v-for="step in steps" :key="step.id" class="flex-1 relative pr-4 last:pr-0">
        <div class="flex items-center gap-3">
          <div
            class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold transition-colors shrink-0"
            :class="{
              'bg-indigo-600 text-white': currentStep === step.id,
              'bg-emerald-600 text-white': currentStep > step.id,
              'bg-slate-200 dark:bg-slate-800 text-slate-500': currentStep < step.id,
            }"
          >
            <span v-if="currentStep > step.id">✓</span>
            <span v-else>{{ step.id }}</span>
          </div>
          <div class="min-w-0">
            <div class="text-xs font-semibold text-slate-900 dark:text-white truncate">{{ step.title }}</div>
            <div v-if="step.subtitle" class="text-[11px] text-slate-400 hidden sm:block truncate">{{ step.subtitle }}</div>
          </div>
        </div>
      </li>
    </ol>
  </nav>
</template>
