<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Terminal,
  Search,
  Pause,
  Play,
  Copy,
  Check,
  ArrowDownToLine,
  Loader2,
} from 'lucide-vue-next'
import type { LogEntry, LogSeverity } from '~/models/generated/workflows.v1'
import { useVirtualList } from '@vueuse/core'

const props = defineProps<{
  logs: LogEntry[]
  isPaused: boolean
  hasMore?: boolean
  isLoadingMore?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:isPaused', val: boolean): void
  (e: 'load-more'): void
}>()

const searchQuery = ref('')
const severityFilter = ref<LogSeverity | 'all'>('all')
const agentFilter = ref<string>('all')
const autoScroll = ref(true)
const copied = ref(false)

const availableAgents = computed(() => {
  const set = new Set<string>()
  for (const l of props.logs) {
    if (l.execution_id) {
      set.add(l.execution_id)
    }
  }
  return Array.from(set).sort()
})

const filteredLogs = computed(() => {
  return props.logs.filter((l) => {
    if (severityFilter.value !== 'all' && l.severity !== severityFilter.value) {
      return false
    }
    if (agentFilter.value !== 'all' && l.execution_id !== agentFilter.value) {
      return false
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      const matchMsg = l.message ? l.message.toLowerCase().includes(q) : false
      const matchComp = l.component ? l.component.toLowerCase().includes(q) : false
      const matchExec = l.execution_id ? l.execution_id.toLowerCase().includes(q) : false
      if (!matchMsg && !matchComp && !matchExec) {
        return false
      }
    }
    return true
  })
})

const { list, containerProps, wrapperProps } = useVirtualList(filteredLogs, {
  itemHeight: 28,
})

const copySelectedLogs = async () => {
  const content = filteredLogs.value
    .map(
      (l) =>
        `[${l.ts || ''}] [${(l.severity || 'info').toUpperCase()}]${l.execution_id ? ` [${l.execution_id}]` : ''} ${l.message}`
    )
    .join('\n')
  try {
    await navigator.clipboard.writeText(content)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
  }
}

const getSeverityClass = (sev: LogSeverity) => {
  switch (sev) {
    case 'error':
      return 'text-rose-500 bg-rose-950/20 font-semibold'
    case 'warn':
      return 'text-amber-400 bg-amber-950/20'
    case 'info':
      return 'text-indigo-400'
    case 'debug':
    default:
      return 'text-slate-500'
  }
}
</script>

<template>
  <div class="h-full flex flex-col bg-slate-950 text-slate-300 font-mono text-xs rounded-lg border border-slate-800 overflow-hidden shadow-inner">

    <div class="p-2.5 bg-slate-900 border-b border-slate-800 flex flex-wrap items-center justify-between gap-3 shrink-0">
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex items-center gap-1.5 text-slate-400">
          <Terminal class="w-4 h-4 text-slate-400" />
          <span class="font-semibold text-slate-200">Terminal Logs</span>
          <span class="text-[10px] text-slate-500">({{ filteredLogs.length }})</span>
        </div>


        <select
          v-model="severityFilter"
          class="bg-slate-800 border border-slate-700 text-slate-300 rounded px-2 py-0.5 text-[11px]"
        >
          <option value="all">All Severities</option>
          <option value="debug">debug</option>
          <option value="info">info</option>
          <option value="warn">warn</option>
          <option value="error">error</option>
        </select>


        <select
          v-if="availableAgents.length > 0"
          v-model="agentFilter"
          class="bg-slate-800 border border-slate-700 text-slate-300 rounded px-2 py-0.5 text-[11px]"
        >
          <option value="all">All Agents</option>
          <option v-for="a in availableAgents" :key="a" :value="a">{{ a }}</option>
        </select>
      </div>

      <div class="flex items-center gap-2">

        <div class="relative w-44 sm:w-56">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search logs..."
            class="w-full bg-slate-800 border border-slate-700 rounded pl-7 pr-2 py-0.5 text-[11px] text-slate-200 placeholder:text-slate-500"
          />
          <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2 top-1.5" />
        </div>


        <button
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition-colors"
          :title="isPaused ? 'Resume stream' : 'Pause stream'"
          @click="emit('update:isPaused', !isPaused)"
        >
          <Play v-if="isPaused" class="w-3.5 h-3.5 text-emerald-400" />
          <Pause v-else class="w-3.5 h-3.5 text-amber-400" />
        </button>


        <button
          class="p-1 rounded border transition-colors"
          :class="autoScroll ? 'bg-indigo-950 text-indigo-400 border-indigo-700' : 'bg-slate-800 text-slate-400 border-slate-700 hover:bg-slate-700'"
          title="Toggle auto-scroll"
          @click="autoScroll = !autoScroll"
        >
          <ArrowDownToLine class="w-3.5 h-3.5" />
        </button>


        <button
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition-colors"
          title="Copy logs"
          @click="copySelectedLogs"
        >
          <Check v-if="copied" class="w-3.5 h-3.5 text-emerald-400" />
          <Copy v-else class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>


    <div v-bind="containerProps" class="flex-1 overflow-y-auto p-2 min-h-[300px]">
      <div v-if="hasMore" class="p-2 text-center">
        <button
          class="px-3 py-1 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-xs border border-slate-700 flex items-center gap-1.5 mx-auto"
          :disabled="isLoadingMore"
          @click="emit('load-more')"
        >
          <Loader2 v-if="isLoadingMore" class="w-3 h-3 animate-spin" />
          <span>Load Prior Logs</span>
        </button>
      </div>

      <div v-if="filteredLogs.length === 0" class="p-8 text-center text-slate-500">
        No log entries recorded.
      </div>

      <div v-else v-bind="wrapperProps">
        <div
          v-for="{ data: log, index } in list"
          :key="log.id || index"
          class="flex items-start gap-2 py-0.5 px-1.5 hover:bg-slate-900/60 rounded text-[11px] leading-relaxed select-text"
          :class="log.severity === 'error' ? 'bg-rose-950/20 text-rose-300' : ''"
        >

          <span class="text-slate-600 shrink-0 w-8 text-right">#{{ log.seq }}</span>
          <span class="text-slate-500 shrink-0">{{ log.ts ? log.ts.slice(11, 23) : '' }}</span>


          <span
            class="px-1 rounded uppercase text-[9px] font-bold shrink-0 tracking-wider"
            :class="getSeverityClass(log.severity || 'info')"
          >
            {{ log.severity }}
          </span>


          <span v-if="log.execution_id" class="text-indigo-400 shrink-0">[{{ log.execution_id }}]</span>
          <span v-if="log.component" class="text-slate-500 shrink-0">({{ log.component }})</span>


          <span class="text-slate-200 break-all flex-1">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
