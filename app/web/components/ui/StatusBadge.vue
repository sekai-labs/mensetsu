<script setup lang="ts">
import type { WorkspaceStatus } from '~/models/common.model'

interface Props {
  status: WorkspaceStatus | string
  size?: 'xs' | 'sm' | 'md'
  showDot?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 'sm',
  showDot: true,
})

const config = computed(() => {
  switch (props.status?.toLowerCase()) {
    case 'ready':
    case 'completed':
      return {
        label: 'Ready',
        dotClass: 'bg-emerald-500',
        badgeClass: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800',
      }
    case 'processing':
    case 'running':
    case 'active':
      return {
        label: 'Processing',
        dotClass: 'bg-indigo-500 animate-pulse',
        badgeClass: 'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800',
      }
    case 'failed':
    case 'error':
      return {
        label: 'Failed',
        dotClass: 'bg-rose-500',
        badgeClass: 'bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800',
      }
    case 'pending':
    default:
      return {
        label: props.status ? props.status.charAt(0).toUpperCase() + props.status.slice(1) : 'Pending',
        dotClass: 'bg-slate-400 dark:bg-slate-500',
        badgeClass: 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700',
      }
  }
})
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 font-medium border rounded-full select-none"
    :class="[
      config.badgeClass,
      size === 'xs' ? 'px-2 py-0.5 text-[10px]' : '',
      size === 'sm' ? 'px-2.5 py-0.5 text-xs' : '',
      size === 'md' ? 'px-3 py-1 text-sm' : '',
    ]"
    role="status"
    :aria-label="`Status: ${config.label}`"
  >
    <span
      v-if="showDot"
      class="rounded-full shrink-0"
      :class="[
        config.dotClass,
        size === 'xs' ? 'w-1.5 h-1.5' : '',
        size === 'sm' ? 'w-1.5 h-1.5' : '',
        size === 'md' ? 'w-2 h-2' : '',
      ]"
      aria-hidden="true"
    />
    <span>{{ config.label }}</span>
  </span>
</template>
