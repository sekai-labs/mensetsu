<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import {
  Clock,
  RotateCcw,
  Bot,
  CheckCircle2,
  AlertCircle,
  Loader2,
  CircleDot,
  Ban,
  PauseCircle,
  Sparkles,
} from 'lucide-vue-next'
import type { TaskStatus } from '~/models/generated/workflows.v1'

export interface TaskNodeData {
  id: string
  name: string
  task_type?: string
  assigned_agent?: string
  status: TaskStatus
  attempt?: number
  duration_ms?: number
  required?: boolean
  added_in_revision?: number
  error?: string | null
}

const props = defineProps<{
  id: string
  data: TaskNodeData
  selected?: boolean
}>()

const statusConfig = computed(() => {
  const s = props.data.status
  switch (s) {
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
    case 'retrying':
      return {
        badgeClass: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/60 dark:text-amber-400 dark:border-amber-800',
        borderClass: 'border-amber-300 dark:border-amber-800',
        icon: RotateCcw,
        iconClass: 'text-amber-600 dark:text-amber-400 animate-spin',
      }
    case 'blocked':
      return {
        badgeClass: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/60 dark:text-amber-400 dark:border-amber-800',
        borderClass: 'border-amber-300 dark:border-amber-800',
        icon: PauseCircle,
        iconClass: 'text-amber-600 dark:text-amber-400',
      }
    case 'cancelled':
    case 'skipped':
      return {
        badgeClass: 'bg-slate-100 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700',
        borderClass: 'border-slate-300 dark:border-slate-700',
        icon: Ban,
        iconClass: 'text-slate-500',
      }
    case 'queued':
    case 'pending':
    default:
      return {
        badgeClass: 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-900 dark:text-slate-400 dark:border-slate-800',
        borderClass: 'border-slate-200 dark:border-slate-800',
        icon: CircleDot,
        iconClass: 'text-slate-400',
      }
  }
})

const isNewInRevision = computed(() => {
  return props.data.added_in_revision && props.data.added_in_revision > 1
})

const isOptional = computed(() => {
  return props.data.required === false
})

const formattedDuration = computed(() => {
  if (!props.data.duration_ms) return null
  const ms = props.data.duration_ms
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
})
</script>

<template>
  <div
    class="w-64 rounded-lg bg-white dark:bg-slate-900 border shadow-xs transition-all duration-150 select-none cursor-pointer"
    :class="[
      statusConfig.borderClass,
      isOptional ? 'border-dashed' : 'border-solid',
      selected ? 'ring-2 ring-indigo-600 dark:ring-indigo-400 shadow-md' : 'hover:border-slate-400 dark:hover:border-slate-600'
    ]"
    data-testid="task-node"
    :data-task-id="id"
    :data-status="data.status"
  >
    <Handle type="target" :position="Position.Top" class="!bg-slate-400 !w-2 !h-2" />

    <div class="p-3 space-y-2">

      <div class="flex items-center justify-between gap-1">
        <div class="flex items-center gap-1.5 min-w-0">
          <span class="text-[10px] font-mono uppercase tracking-wider text-slate-400 truncate max-w-[90px]">
            {{ data.task_type || 'task' }}
          </span>

          <span
            v-if="isNewInRevision"
            class="inline-flex items-center gap-0.5 px-1 py-0.2 rounded text-[9px] font-mono font-medium bg-amber-50 dark:bg-amber-950/60 text-amber-700 dark:text-amber-400 border border-amber-200 dark:border-amber-800"
            data-testid="rev-badge"
            :title="`Added dynamically in plan revision ${data.added_in_revision}`"
          >
            <Sparkles class="w-2.5 h-2.5 text-amber-500" />
            <span>rev {{ data.added_in_revision }}</span>
          </span>
          <span
            v-if="isOptional"
            class="text-[9px] font-mono text-slate-400 px-1 py-0.2 rounded border border-slate-200 dark:border-slate-800"
          >
            opt
          </span>
        </div>

        <span
          class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium border"
          :class="statusConfig.badgeClass"
          data-testid="task-status-badge"
        >
          <component :is="statusConfig.icon" class="w-3 h-3 shrink-0" :class="statusConfig.iconClass" />
          <span class="capitalize">{{ data.status }}</span>
        </span>
      </div>


      <div class="text-xs font-semibold text-slate-900 dark:text-white line-clamp-2 leading-tight">
        {{ data.name }}
      </div>


      <div class="pt-2 border-t border-slate-100 dark:border-slate-800/80 flex items-center justify-between gap-2 text-[11px] text-slate-500">

        <div class="flex items-center gap-1 min-w-0">
          <Bot class="w-3.5 h-3.5 text-indigo-500 shrink-0" />
          <span class="truncate font-mono text-[10px] text-slate-600 dark:text-slate-300">
            {{ data.assigned_agent || 'Unassigned' }}
          </span>
        </div>

        <div class="flex items-center gap-2 shrink-0">

          <span
            v-if="data.attempt && data.attempt > 1"
            class="inline-flex items-center gap-0.5 text-[10px] font-mono text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/60 px-1 py-0.2 rounded border border-amber-200 dark:border-amber-800"
            title="Attempt count"
          >
            <RotateCcw class="w-2.5 h-2.5" />
            <span>att {{ data.attempt }}</span>
          </span>


          <span
            v-if="formattedDuration"
            class="flex items-center gap-1 font-mono text-[10px] text-slate-400"
          >
            <Clock class="w-3 h-3 text-slate-400" />
            <span>{{ formattedDuration }}</span>
          </span>
        </div>
      </div>
    </div>

    <Handle type="source" :position="Position.Bottom" class="!bg-slate-400 !w-2 !h-2" />
  </div>
</template>
