<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  status: 'pending' | 'processing' | 'ready' | 'failed' | string
  size?: 'sm' | 'md'
}>()

const badgeClass = computed(() => {
  const base = props.size === 'sm' ? 'px-2 py-0.5 text-[11px]' : 'px-2.5 py-1 text-xs'
  switch (props.status.toLowerCase()) {
    case 'ready':
      return `${base} bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800`
    case 'processing':
      return `${base} bg-blue-50 dark:bg-blue-950/50 text-blue-700 dark:text-blue-400 border border-blue-200 dark:border-blue-800 animate-pulse`
    case 'failed':
      return `${base} bg-rose-50 dark:bg-rose-950/50 text-rose-700 dark:text-rose-400 border border-rose-200 dark:border-rose-800`
    default:
      return `${base} bg-amber-50 dark:bg-amber-950/50 text-amber-700 dark:text-amber-400 border border-amber-200 dark:border-amber-800`
  }
})
</script>

<template>
  <span :class="['inline-flex items-center rounded font-semibold uppercase tracking-wider', badgeClass]">
    <slot>{{ status }}</slot>
  </span>
</template>
