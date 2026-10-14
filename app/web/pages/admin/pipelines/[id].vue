<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useAdminService } from '~/features/admin/services/admin.service'
import {
  DEFAULT_PIPELINE_STEPS,
  calculateStepState,
  getStepCardStyle,
  getLogLevelBadgeStyle,
  filterPipelineLogs,
  formatLogsForExport,
} from '~/features/admin/utils/admin.util'

definePageMeta({
  layout: 'default',
})

const route = useRoute()
const pipelineId = computed(() => String(route.params.id))
const { getPipelineDetail, getPipelineLogs } = useAdminService()

const { data: detail } = await useAsyncData(
  `admin-pipe-${pipelineId.value}`,
  () => getPipelineDetail(pipelineId.value)
)
const { data: logsData } = await useAsyncData(
  `admin-pipe-logs-${pipelineId.value}`,
  () => getPipelineLogs(pipelineId.value)
)

const logFilterQuery = ref('')
const logLevel = ref('all')

const filteredLogs = computed(() => {
  return filterPipelineLogs(logsData.value?.logs || [], logFilterQuery.value, logLevel.value)
})

const copyLogs = async () => {
  if (logsData.value?.logs) {
    const text = formatLogsForExport(logsData.value.logs, pipelineId.value)
    await navigator.clipboard.writeText(text)
    alert('Logs copied to clipboard.')
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <NuxtLink to="/admin/pipelines" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium inline-block mb-1">
          ← Back to Pipelines
        </NuxtLink>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Pipeline Telemetry & Execution Stepper
        </h1>
        <p class="text-xs text-slate-500 font-mono mt-0.5">
          Workspace ID: {{ pipelineId }}
        </p>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="px-3 py-1.5 text-xs font-medium rounded border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 transition-colors"
          @click="copyLogs"
        >
          Export Logs Text
        </button>
      </div>
    </div>


    <div class="space-y-3">
      <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">
        Temporal Workflow Execution Phases
      </h2>
      <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 gap-2">
        <div
          v-for="step in DEFAULT_PIPELINE_STEPS"
          :key="step.id"
          class="p-2.5 rounded border text-xs space-y-1 transition-colors"
          :class="getStepCardStyle(calculateStepState(step.id, detail?.workspace?.current_phase || '', detail?.workspace?.status || ''))"
        >
          <div class="font-semibold truncate">{{ step.name }}</div>
          <div class="text-[10px] text-slate-500 font-mono capitalize">
            {{ calculateStepState(step.id, detail?.workspace?.current_phase || '', detail?.workspace?.status || '') }}
          </div>
        </div>
      </div>
    </div>


    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">
          Activity Execution Logs
        </h2>
        <div class="flex items-center gap-1.5 text-xs">
          <button
            v-for="lvl in ['all', 'info', 'warn', 'error']"
            :key="lvl"
            type="button"
            class="px-2 py-0.5 rounded capitalize font-medium text-[11px]"
            :class="logLevel === lvl ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900' : 'text-slate-500 hover:text-slate-900'"
            @click="logLevel = lvl"
          >
            {{ lvl }}
          </button>
        </div>
      </div>

      <div class="bg-slate-900 text-slate-200 rounded-lg p-4 font-mono text-xs space-y-2 overflow-x-auto max-h-96 overflow-y-auto">
        <div v-if="!filteredLogs.length" class="text-slate-500 text-center py-4">
          No logs matching level criteria.
        </div>
        <div
          v-for="(log, idx) in filteredLogs"
          :key="idx"
          class="flex items-start gap-3 py-1 border-b border-slate-800 last:border-0"
        >
          <span class="text-slate-500 shrink-0 text-[11px]">{{ log.timestamp }}</span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold shrink-0"
            :class="getLogLevelBadgeStyle(log.level)"
          >
            {{ log.level }}
          </span>
          <span class="text-indigo-400 shrink-0 text-[11px] font-semibold">[{{ log.step }}]</span>
          <span class="text-slate-300 break-all">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
