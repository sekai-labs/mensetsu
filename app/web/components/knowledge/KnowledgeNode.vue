<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import { getSkillNodeLabel, getSkillNodeStatusCategory } from '~/features/interviews/utils/interview.util'
import type { KnowledgeSkillNode } from '~/features/interviews/models/interview.model'

interface Props {
  id: string
  data: KnowledgeSkillNode
  selected?: boolean
}

const props = defineProps<Props>()

const label = computed(() => getSkillNodeLabel(props.data))
const statusCategory = computed(() => getSkillNodeStatusCategory(props.data))
</script>

<template>
  <div
    class="px-3.5 py-2.5 rounded-lg border shadow-sm transition-all text-xs font-mono select-none min-w-[200px] max-w-[260px] cursor-pointer"
    :class="[
      statusCategory === 'matched'
        ? 'bg-emerald-50/90 dark:bg-emerald-950/80 border-emerald-300 dark:border-emerald-700 text-emerald-950 dark:text-emerald-100 hover:border-emerald-500'
        : (statusCategory === 'in_progress'
          ? 'bg-sky-50/90 dark:bg-sky-950/80 border-sky-300 dark:border-sky-700 text-sky-950 dark:text-sky-100 hover:border-sky-500'
          : 'bg-amber-50/90 dark:bg-amber-950/80 border-amber-300 dark:border-amber-700 text-amber-950 dark:text-amber-100 hover:border-amber-500'),
      selected ? 'ring-2 ring-indigo-500 ring-offset-1 dark:ring-offset-slate-900 shadow-md' : '',
    ]"
  >
    <Handle type="target" :position="Position.Top" class="!bg-indigo-500 !w-2 !h-2" />

    <div class="flex items-center justify-between gap-2 mb-1.5">
      <span class="font-bold truncate text-[11px] leading-tight" :title="label">
        {{ label }}
      </span>
      <span
        class="text-[9px] uppercase px-1.5 py-0.2 rounded font-bold shrink-0"
        :class="{
          'bg-emerald-200/80 dark:bg-emerald-900/80 text-emerald-800 dark:text-emerald-200': statusCategory === 'matched',
          'bg-sky-200/80 dark:bg-sky-900/80 text-sky-800 dark:text-sky-200': statusCategory === 'in_progress',
          'bg-amber-200/80 dark:bg-amber-900/80 text-amber-800 dark:text-amber-200': statusCategory === 'gap',
        }"
      >
        {{ statusCategory === 'matched' ? 'MATCH' : (statusCategory === 'in_progress' ? 'PROGRESS' : 'GAP') }}
      </span>
    </div>

    <div class="flex items-center justify-between text-[10px] opacity-75 pt-1 border-t border-current/15">
      <span class="capitalize truncate max-w-[110px]">{{ (props.data.category || 'general').replace(/_/g, ' ') }}</span>
      <span class="capitalize text-[9px] shrink-0">{{ props.data.importance || 'core' }}</span>
    </div>

    <Handle type="source" :position="Position.Bottom" class="!bg-indigo-500 !w-2 !h-2" />
  </div>
</template>
