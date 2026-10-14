<script setup lang="ts">
import { computed } from 'vue'
interface Props {
  tier: 'FACT' | 'INFERENCE' | 'CANDIDATE' | 'GAP' | string
  size?: 'xs' | 'sm'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'xs',
})

const style = computed(() => {
  switch (props.tier?.toUpperCase()) {
    case 'FACT':
      return 'bg-emerald-50 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800'
    case 'INFERENCE':
      return 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800'
    case 'CANDIDATE':
      return 'bg-amber-50 dark:bg-amber-950/60 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800'
    case 'GAP':
      return 'bg-rose-50 dark:bg-rose-950/60 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800'
    default:
      return 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700'
  }
})
</script>

<template>
  <span
    class="inline-flex items-center font-mono font-bold uppercase tracking-wider border rounded select-none"
    :class="[
      style,
      size === 'xs' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-xs',
    ]"
  >
    [{{ tier }}]
  </span>
</template>
