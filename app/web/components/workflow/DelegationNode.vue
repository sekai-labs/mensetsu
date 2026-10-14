<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import { CheckCircle2, AlertCircle, Loader2, CircleDot, Bot } from 'lucide-vue-next'
import type { DelegationGraphNodeData } from '~/models/generated/workflows.v1'

const props = defineProps<{
  id: string
  label?: string
  data: DelegationGraphNodeData
  selected?: boolean
}>()

const statusConfig = computed(() => {
  switch (props.data.status) {
    case 'completed':
      return {
        badgeClass: 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-400 dark:border-emerald-800',
        borderClass: 'border-emerald-300 dark:border-emerald-800',
        icon: CheckCircle2,
        iconClass: 'text-emerald-600 dark:text-emerald-400',
      }
    case 'running':
      return {
        badgeClass: 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/60 dark:text-indigo-400 dark:border-indigo-800',
        borderClass: 'border-indigo-400 dark:border-indigo-600 ring-2 ring-indigo-500/20',
        icon: Loader2,
        iconClass: 'text-indigo-600 dark:text-indigo-400 animate-spin',
      }
    case 'failed':
      return {
        badgeClass: 'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/60 dark:text-rose-400 dark:border-rose-800',
        borderClass: 'border-rose-300 dark:border-rose-800',
        icon: AlertCircle,
        iconClass: 'text-rose-600 dark:text-rose-400',
      }
    default:
      return {
        badgeClass: 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-900 dark:text-slate-400 dark:border-slate-800',
        borderClass: 'border-slate-200 dark:border-slate-800',
        icon: CircleDot,
        iconClass: 'text-slate-400',
      }
  }
})
</script>

<template>
  <div
    class="w-64 rounded-lg bg-white dark:bg-slate-900 border shadow-xs transition-all duration-150 select-none cursor-pointer"
    :class="[
      statusConfig.borderClass,
      selected ? 'ring-2 ring-indigo-600 dark:ring-indigo-400 shadow-md' : 'hover:border-slate-400 dark:hover:border-slate-600'
    ]"
    data-testid="delegation-node"
    :data-agent-id="data.agent_id"
  >
    <Handle type="target" :position="Position.Left" class="!bg-slate-400 !w-2 !h-2" />

    <div class="p-3 space-y-2">

      <div class="flex items-center justify-between gap-1">
        <span class="inline-flex items-center gap-1 text-[10px] font-mono uppercase tracking-wider text-indigo-600 dark:text-indigo-400">
          <Bot class="w-3.5 h-3.5" />
          <span>Agent Node</span>
        </span>
        <span
          class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium border"
          :class="statusConfig.badgeClass"
        >
          <component :is="statusConfig.icon" class="w-3 h-3 shrink-0" :class="statusConfig.iconClass" />
          <span class="capitalize">{{ data.status }}</span>
        </span>
      </div>


      <div class="text-xs font-mono font-semibold text-slate-900 dark:text-white truncate">
        {{ label || data.agent_id }}
      </div>


      <div class="pt-2 border-t border-slate-100 dark:border-slate-800/80 text-[11px] text-slate-500 font-mono truncate">
        Task: {{ data.task_id }}
      </div>
    </div>

    <Handle type="source" :position="Position.Right" class="!bg-slate-400 !w-2 !h-2" />
  </div>
</template>
