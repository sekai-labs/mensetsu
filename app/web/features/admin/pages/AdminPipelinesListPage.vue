<script setup lang="ts">
import { computed, ref } from 'vue'
import type { AdminPipelineItem } from '../models/admin.model'
import { useAdminService } from '../services/admin.service'
import PipelineStatusPill from '../components/PipelineStatusPill.vue'

const { listAdminPipelines } = useAdminService()

const { data, pending, refresh, error } = await useAsyncData(
  'admin-pipelines-list',
  () => listAdminPipelines()
)

const filterText = ref('')
const selectedStatus = ref<string>('all')

const allPipelines = computed<AdminPipelineItem[]>(() => data.value?.pipelines || [])

const filteredPipelines = computed<AdminPipelineItem[]>(() => {
  const query = filterText.value.trim().toLowerCase()
  const statusFilter = selectedStatus.value.toLowerCase()

  return allPipelines.value.filter((pipeline) => {
    const matchesQuery =
      !query ||
      pipeline.company_name?.toLowerCase().includes(query) ||
      pipeline.role_title?.toLowerCase().includes(query) ||
      pipeline.workspace_id?.toLowerCase().includes(query)

    const matchesStatus =
      statusFilter === 'all' ||
      !statusFilter ||
      pipeline.status?.toLowerCase() === statusFilter

    return matchesQuery && matchesStatus
  })
})

const getProgressBarColor = (status: string): string => {
  switch (status?.toLowerCase()) {
    case 'ready':
      return 'bg-emerald-600'
    case 'failed':
      return 'bg-rose-500'
    case 'processing':
      return 'bg-indigo-600 animate-pulse'
    default:
      return 'bg-slate-400'
  }
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-5 border-b border-slate-200 dark:border-slate-800">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono uppercase tracking-wider text-indigo-600 dark:text-indigo-400 font-bold mb-1">
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-ping"></span>
          <span>Operations & Observability</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Admin Pipeline Orchestration Monitor
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Real-time visibility into durable background pipeline executions, step progression, and streaming activity traces.
        </p>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          :disabled="pending"
          @click="refresh()"
          class="px-3.5 py-2 text-xs font-semibold rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-750 transition flex items-center gap-2 shadow-sm disabled:opacity-50"
        >
          <span :class="{ 'animate-spin': pending }">↻</span>
          <span>Refresh Executions</span>
        </button>
      </div>
    </div>

    <!-- Filters & Summary Bar -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-lg bg-slate-50 dark:bg-slate-850/60 border border-slate-200 dark:border-slate-800 text-xs">
      <div class="flex flex-wrap items-center gap-3">
        <input
          v-model="filterText"
          type="text"
          placeholder="Filter by company, role, or ID..."
          class="px-3 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-64 text-xs"
        />

        <select
          v-model="selectedStatus"
          class="px-3 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 focus:outline-none text-xs cursor-pointer"
        >
          <option value="all">All Statuses</option>
          <option value="ready">Ready (Completed)</option>
          <option value="processing">Processing</option>
          <option value="failed">Failed</option>
          <option value="pending">Pending</option>
        </select>
      </div>

      <div class="text-slate-500 dark:text-slate-400 font-mono text-[11px]">
        Showing {{ filteredPipelines.length }} of {{ allPipelines.length }} pipelines
      </div>
    </div>

    <!-- Error State -->
    <div
      v-if="error"
      class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs"
    >
      Failed to load pipeline executions: {{ error.message || 'Unknown network error' }}
    </div>

    <!-- Loading Skeleton -->
    <div v-if="pending" class="space-y-3">
      <div v-for="i in 5" :key="i" class="h-16 bg-slate-100 dark:bg-slate-850 rounded-lg animate-pulse"></div>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="filteredPipelines.length === 0"
      class="p-12 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg"
    >
      No pipeline executions match the current search filters.
    </div>

    <!-- Data Table -->
    <div v-else class="overflow-x-auto border border-slate-200 dark:border-slate-800 rounded-lg bg-white dark:bg-slate-900 shadow-sm">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-50 dark:bg-slate-850 text-slate-500 uppercase tracking-wider font-semibold border-b border-slate-200 dark:border-slate-800">
          <tr>
            <th class="py-3 px-4">Workspace / Target</th>
            <th class="py-3 px-4">Status</th>
            <th class="py-3 px-4">Current Phase</th>
            <th class="py-3 px-4">Progress</th>
            <th class="py-3 px-4">Execution Steps</th>
            <th class="py-3 px-4 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
          <tr
            v-for="p in filteredPipelines"
            :key="p.workspace_id"
            class="hover:bg-slate-50/70 dark:hover:bg-slate-850/50 transition"
          >
            <td class="py-3.5 px-4">
              <div class="font-bold text-slate-900 dark:text-white">{{ p.company_name }}</div>
              <div class="text-[11px] text-slate-500">{{ p.role_title }}</div>
              <div class="text-[10px] font-mono text-slate-400 mt-0.5 truncate max-w-xs">{{ p.workspace_id }}</div>
            </td>
            <td class="py-3.5 px-4">
              <PipelineStatusPill :status="p.status" />
            </td>
            <td class="py-3.5 px-4 font-mono text-slate-700 dark:text-slate-300 text-[11px]">
              {{ p.current_phase?.replace(/_/g, ' ') }}
            </td>
            <td class="py-3.5 px-4">
              <div class="w-32 space-y-1">
                <div class="flex items-center justify-between text-[10px] text-slate-500 font-mono">
                  <span>{{ p.progress_percent }}%</span>
                </div>
                <div class="w-full bg-slate-100 dark:bg-slate-800 h-1.5 rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-300"
                    :class="getProgressBarColor(p.status)"
                    :style="{ width: `${p.progress_percent}%` }"
                  ></div>
                </div>
              </div>
            </td>
            <td class="py-3.5 px-4 font-mono text-[11px] text-slate-600 dark:text-slate-400">
              <span v-if="p.execution">
                {{ p.execution.completed_steps || 0 }} / {{ p.execution.total_steps || 8 }} steps
              </span>
              <span v-else class="text-slate-400">
                Queued
              </span>
            </td>
            <td class="py-3.5 px-4 text-right">
              <NuxtLink
                :to="`/admin/pipelines/${p.workspace_id}`"
                class="px-2.5 py-1 text-xs font-semibold rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 dark:hover:bg-indigo-900 transition"
              >
                Inspect Logs →
              </NuxtLink>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
