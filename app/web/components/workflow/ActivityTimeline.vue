<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Clock,
  Play,
  CheckCircle2,
  Bot,
  Wrench,
  GitFork,
  FileText,
  Filter,
  Search,
} from 'lucide-vue-next'
import type { WorkflowEvent } from '~/models/generated/workflows.v1'

const props = defineProps<{
  events: WorkflowEvent[]
}>()

const emit = defineEmits<{
  (e: 'select-task', taskId: string): void
  (e: 'select-execution', executionId: string): void
  (e: 'select-artifact', artifactId: string): void
}>()

const selectedTypeFilter = ref<string>('all')
const selectedAgentFilter = ref<string>('all')
const searchQuery = ref<string>('')

const availableTypes = computed(() => {
  const set = new Set<string>()
  for (const ev of props.events) {
    if (ev.type) set.add(ev.type)
  }
  return Array.from(set).sort()
})

const availableAgents = computed(() => {
  const set = new Set<string>()
  for (const ev of props.events) {
    const payload = ev.payload as unknown as Record<string, unknown>
    if (typeof payload?.assigned_agent === 'string') {
      set.add(payload.assigned_agent)
    } else if (typeof payload?.agent_id === 'string') {
      set.add(payload.agent_id)
    }
  }
  return Array.from(set).sort()
})

const filteredEvents = computed(() => {
  return props.events.filter((ev) => {
    if (selectedTypeFilter.value !== 'all' && ev.type !== selectedTypeFilter.value) {
      return false
    }

    const payload = ev.payload as unknown as Record<string, unknown>
    const agent = (payload?.assigned_agent || payload?.agent_id) as string | undefined
    if (selectedAgentFilter.value !== 'all' && agent !== selectedAgentFilter.value) {
      return false
    }

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      const matchType = ev.type ? ev.type.toLowerCase().includes(q) : false
      const matchTaskId = ev.task_id?.toLowerCase().includes(q)
      const matchExecId = ev.execution_id?.toLowerCase().includes(q)
      const matchPayload = JSON.stringify(ev.payload).toLowerCase().includes(q)
      if (!matchType && !matchTaskId && !matchExecId && !matchPayload) {
        return false
      }
    }

    return true
  })
})

const getEventIcon = (type: string) => {
  if (type.startsWith('workflow.')) return Play
  if (type.startsWith('task.')) return CheckCircle2
  if (type.startsWith('agent.')) return Bot
  if (type.startsWith('tool.')) return Wrench
  if (type.startsWith('delegation.')) return GitFork
  if (type.startsWith('artifact.')) return FileText
  return Clock
}

const getEventColorClass = (type: string) => {
  if (type.includes('failed')) return 'text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-950/40 border-rose-200 dark:border-rose-800'
  if (type.includes('completed')) return 'text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-800'
  if (type.includes('retrying') || type.includes('blocked')) return 'text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/40 border-amber-200 dark:border-amber-800'
  return 'text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/40 border-indigo-200 dark:border-indigo-800'
}

const handleEventClick = (ev: WorkflowEvent) => {
  if (ev.task_id) {
    emit('select-task', ev.task_id)
  } else if (ev.execution_id) {
    emit('select-execution', ev.execution_id)
  }
}
</script>

<template>
  <div class="space-y-4">

    <div class="flex flex-wrap items-center justify-between gap-3 p-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg">
      <div class="flex flex-wrap items-center gap-3">

        <div class="flex items-center gap-1.5 text-xs text-slate-500">
          <Filter class="w-3.5 h-3.5" />
          <select
            v-model="selectedTypeFilter"
            class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-2 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono"
          >
            <option value="all">All Types ({{ events.length }})</option>
            <option v-for="t in availableTypes" :key="t" :value="t">{{ t }}</option>
          </select>
        </div>


        <div v-if="availableAgents.length > 0" class="flex items-center gap-1.5 text-xs text-slate-500">
          <Bot class="w-3.5 h-3.5" />
          <select
            v-model="selectedAgentFilter"
            class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-2 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono"
          >
            <option value="all">All Agents</option>
            <option v-for="a in availableAgents" :key="a" :value="a">{{ a }}</option>
          </select>
        </div>
      </div>


      <div class="w-full sm:w-64 relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter activity..."
          class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded pl-8 pr-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono placeholder:text-slate-400"
        />
        <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-2" />
      </div>
    </div>


    <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden divide-y divide-slate-100 dark:divide-slate-800/80">
      <div v-if="filteredEvents.length === 0" class="p-8 text-center text-xs text-slate-400 font-mono">
        No execution activity recorded matching criteria.
      </div>

      <div
        v-for="ev in filteredEvents"
        :key="ev.event_id || ev.seq"
        class="p-3 flex items-start gap-3 hover:bg-slate-50/70 dark:hover:bg-slate-800/40 transition-colors cursor-pointer text-xs"
        :data-seq="ev.seq"
        data-testid="timeline-event"
        @click="handleEventClick(ev)"
      >
        <div class="shrink-0 flex flex-col items-end w-12 font-mono text-[10px] text-slate-400">
          <span>#{{ ev.seq }}</span>
          <span class="text-[9px]">{{ ev.ts ? ev.ts.slice(11, 19) : '' }}</span>
        </div>
        <div
          class="w-6 h-6 rounded flex items-center justify-center shrink-0 border"
          :class="getEventColorClass(ev.type || '')"
        >
          <component :is="getEventIcon(ev.type || '')" class="w-3.5 h-3.5" />
        </div>

        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="font-mono font-semibold text-slate-900 dark:text-white">{{ ev.type }}</span>
            <span v-if="ev.task_id" class="text-[10px] font-mono text-slate-500">task: {{ ev.task_id }}</span>
            <span v-if="ev.execution_id" class="text-[10px] font-mono text-slate-500">exec: {{ ev.execution_id }}</span>
          </div>

          <div class="text-[11px] text-slate-600 dark:text-slate-400 font-mono mt-0.5 truncate">
            {{ JSON.stringify(ev.payload) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
