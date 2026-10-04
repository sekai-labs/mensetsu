<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import type { AdminPipelineLog, StepState } from '../models/admin.model'
import { useAdminService } from '../services/admin.service'
import PipelineStatusPill from '../components/PipelineStatusPill.vue'
import PipelineStepper from '../components/PipelineStepper.vue'
import PipelineLogViewer from '../components/PipelineLogViewer.vue'
import { formatLogsForExport } from '../utils/admin.util'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id || ''))

const { getAdminPipelineDetail, getAdminPipelineLogs } = useAdminService()

const {
  data: detailData,
  refresh: refreshDetail,
  error: detailError,
} = await useAsyncData(
  `admin-pipeline-detail-${workspaceId.value}`,
  () => getAdminPipelineDetail(workspaceId.value)
)

const {
  data: logsData,
  refresh: refreshLogs,
  error: logsError,
} = await useAsyncData(
  `admin-pipeline-logs-${workspaceId.value}`,
  () => getAdminPipelineLogs(workspaceId.value)
)

const logs = ref<AdminPipelineLog[]>([...(logsData.value?.logs || [])])
const isSseConnected = ref(false)
let eventSource: EventSource | null = null

watch(
  () => logsData.value?.logs,
  (newLogs) => {
    if (newLogs && logs.value.length === 0) {
      logs.value = [...newLogs]
    }
  }
)

const currentPhase = computed(() => {
  return detailData.value?.workspace?.current_phase || 'preparing_input'
})

const status = computed(() => {
  return detailData.value?.workspace?.status || 'pending'
})

const progressPercent = computed(() => {
  return detailData.value?.workspace?.progress_percent || 0
})

const companyName = computed(() => {
  return detailData.value?.workspace?.company_name || 'Workspace'
})

const roleTitle = computed(() => {
  return detailData.value?.workspace?.role_title || ''
})

const connectSse = () => {
  if (import.meta.server) return

  const cleanId = workspaceId.value.replace(/^workspace:/, '').replace(/`/g, '')
  const url = `/api/backend/admin/pipelines/${cleanId}/events`
  eventSource = new EventSource(url, { withCredentials: true })

  eventSource.onopen = () => {
    isSseConnected.value = true
  }

  eventSource.onerror = () => {
    isSseConnected.value = false
  }

  eventSource.addEventListener('log', (event: MessageEvent) => {
    try {
      const parsed: unknown = JSON.parse(event.data)
      if (parsed && typeof parsed === 'object' && 'message' in parsed && 'step' in parsed) {
        logs.value.push(parsed as AdminPipelineLog)
      }
    } catch {
      // Ignore JSON parse errors in SSE stream
    }
  })

  eventSource.addEventListener('status', (event: MessageEvent) => {
    try {
      const parsed: unknown = JSON.parse(event.data)
      if (parsed && typeof parsed === 'object' && detailData.value?.workspace) {
        const update = parsed as {
          status?: string
          current_phase?: string
          progress_percent?: number
        }
        if (update.status) detailData.value.workspace.status = update.status as typeof detailData.value.workspace.status
        if (update.current_phase) detailData.value.workspace.current_phase = update.current_phase as typeof detailData.value.workspace.current_phase
        if (typeof update.progress_percent === 'number') {
          detailData.value.workspace.progress_percent = update.progress_percent
        }
      }
    } catch {
      // Ignore JSON parse errors in SSE stream
    }
  })
}

const refreshAll = async () => {
  await Promise.all([refreshDetail(), refreshLogs()])
  if (logsData.value?.logs) {
    logs.value = [...logsData.value.logs]
  }
}

const downloadLogs = () => {
  const content = formatLogsForExport(logs.value, workspaceId.value)
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `pipeline-${workspaceId.value}.log`
  a.click()
  URL.revokeObjectURL(url)
}

onMounted(() => {
  connectSse()
})

onUnmounted(() => {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }
  isSseConnected.value = false
})
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-5 border-b border-slate-200 dark:border-slate-800">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-slate-500 mb-1">
          <NuxtLink to="/admin/pipelines" class="hover:text-indigo-600 transition flex items-center gap-1">
            <span>←</span>
            <span>All Pipelines</span>
          </NuxtLink>
          <span>/</span>
          <span class="text-slate-900 dark:text-white font-bold">{{ companyName }}</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white flex items-center gap-3">
          <span>Pipeline: {{ companyName }} <span v-if="roleTitle">— {{ roleTitle }}</span></span>
          <PipelineStatusPill :status="status" size="md" />
        </h1>
        <p class="text-xs font-mono text-slate-400 mt-1">ID: {{ workspaceId }}</p>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          @click="downloadLogs"
          class="px-3 py-1.5 text-xs font-semibold rounded border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition"
        >
          Download Logs (.log)
        </button>
        <button
          type="button"
          @click="refreshAll"
          class="px-3 py-1.5 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
        >
          Refresh State
        </button>
      </div>
    </div>

    <!-- Error Banner -->
    <div
      v-if="detailError || logsError"
      class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs"
    >
      Failed to fetch latest pipeline state: {{ detailError?.message || logsError?.message }}
    </div>

    <!-- Execution Stepper -->
    <PipelineStepper
      :current-phase="currentPhase"
      :status="status"
      :progress-percent="progressPercent"
    />

    <!-- Real-Time Terminal Log Stream -->
    <PipelineLogViewer
      :logs="logs"
      :workspace-id="workspaceId"
      :is-connected="isSseConnected"
      @refresh="refreshLogs"
    />
  </div>
</template>
