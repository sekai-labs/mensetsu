<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue'
import type { AdminPipelineLog } from '../models/admin.model'
import {
  filterPipelineLogs,
  formatLogTimestamp,
  formatLogsForExport,
  getLogLevelBadgeStyle,
} from '../utils/admin.util'

const props = withDefaults(
  defineProps<{
    logs: AdminPipelineLog[]
    workspaceId: string
    isConnected?: boolean
  }>(),
  {
    isConnected: true,
  }
)

const emit = defineEmits<{
  (e: 'refresh'): void
}>()

const logSearch = ref('')
const selectedLogLevel = ref('all')
const autoScroll = ref(true)
const logContainer = ref<HTMLDivElement | null>(null)

const filteredLogs = computed(() => {
  return filterPipelineLogs(props.logs, logSearch.value, selectedLogLevel.value)
})

const scrollToBottom = () => {
  if (autoScroll.value && logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

watch(
  () => props.logs.length,
  async () => {
    await nextTick()
    scrollToBottom()
  }
)

const handleDownload = () => {
  const content = formatLogsForExport(props.logs, props.workspaceId)
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = `pipeline-${props.workspaceId}.log`
  anchor.click()
  URL.revokeObjectURL(url)
}

defineExpose({
  scrollToBottom,
})
</script>

<template>
  <div class="rounded-lg border border-slate-800 bg-slate-950 shadow-md overflow-hidden text-slate-200 font-mono text-xs">
    <!-- Log Viewer Toolbar -->
    <div class="bg-slate-900 px-4 py-3 border-b border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded-full bg-rose-500"></div>
        <div class="w-3 h-3 rounded-full bg-amber-500"></div>
        <div class="w-3 h-3 rounded-full bg-emerald-500"></div>
        <span class="ml-2 text-xs font-semibold text-slate-300">Live Activity Log Stream (SSE)</span>
        <span
          class="text-[10px] px-2 py-0.5 rounded font-bold uppercase tracking-wider"
          :class="
            isConnected
              ? 'bg-slate-800 text-emerald-400 border border-emerald-900/50'
              : 'bg-slate-800 text-slate-400 border border-slate-700'
          "
        >
          {{ isConnected ? 'CONNECTED' : 'DISCONNECTED' }}
        </span>
        <span class="text-[11px] text-slate-400 font-normal">
          ({{ filteredLogs.length }} / {{ logs.length }} events)
        </span>
      </div>

      <div class="flex items-center gap-3">
        <input
          v-model="logSearch"
          type="text"
          placeholder="Filter logs..."
          class="px-2.5 py-1 rounded bg-slate-800 border border-slate-700 text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-44"
        />

        <select
          v-model="selectedLogLevel"
          class="px-2 py-1 rounded bg-slate-800 border border-slate-700 text-xs text-slate-300 focus:outline-none cursor-pointer"
        >
          <option value="all">All Levels</option>
          <option value="info">Info</option>
          <option value="warn">Warn</option>
          <option value="error">Error</option>
          <option value="debug">Debug</option>
        </select>

        <label class="flex items-center gap-1.5 text-xs text-slate-400 cursor-pointer select-none">
          <input
            v-model="autoScroll"
            type="checkbox"
            class="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-0"
          />
          <span>Auto-scroll</span>
        </label>

        <button
          type="button"
          @click="handleDownload"
          class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-750 text-slate-300 hover:text-white border border-slate-700 transition"
          title="Download complete logs"
        >
          Export
        </button>

        <button
          type="button"
          @click="emit('refresh')"
          class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-750 text-slate-300 hover:text-white border border-slate-700 transition"
          title="Refresh server logs"
        >
          ↻
        </button>
      </div>
    </div>

    <!-- Log Lines Container -->
    <div
      ref="logContainer"
      class="p-4 h-96 overflow-y-auto space-y-1.5 select-text selection:bg-indigo-600 selection:text-white"
    >
      <div v-if="filteredLogs.length === 0" class="text-slate-500 italic py-12 text-center font-sans text-xs">
        No log events matching filters received. Activity will stream here in real time.
      </div>

      <div
        v-for="(log, idx) in filteredLogs"
        :key="log.id || `${log.timestamp}-${idx}`"
        class="flex items-start gap-2 hover:bg-slate-900/80 px-1 py-0.5 rounded leading-relaxed"
      >
        <span class="text-slate-500 shrink-0 text-[11px] tabular-nums">
          {{ formatLogTimestamp(log.timestamp) }}
        </span>

        <span
          :class="[
            'px-1.5 py-0.2 rounded text-[10px] font-bold shrink-0 uppercase border',
            getLogLevelBadgeStyle(log.level),
          ]"
        >
          {{ log.level || 'INFO' }}
        </span>

        <span class="text-indigo-400 shrink-0 font-medium">
          [{{ log.step }}]
        </span>

        <span class="text-slate-300 break-all flex-1">
          {{ log.message }}
        </span>
      </div>
    </div>
  </div>
</template>
