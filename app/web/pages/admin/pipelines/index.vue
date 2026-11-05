<script setup lang="ts">
import { useAdminService } from '~/features/admin/services/admin.service'
import StatusBadge from '~/components/ui/StatusBadge.vue'
import TokenUsagePanel from '~/components/usage/TokenUsagePanel.vue'

definePageMeta({
  layout: 'default',
})

const { listPipelines } = useAdminService()
const { data: pipelinesData } = await useAsyncData('admin-pipelines', () => listPipelines())
</script>

<template>
  <div class="space-y-6">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 mb-1">
        <span class="inline-flex items-center rounded border border-indigo-200 bg-indigo-50 px-2 py-1 font-mono text-[10px] font-semibold uppercase tracking-[0.12em] text-indigo-800 dark:border-indigo-900 dark:bg-indigo-950/40 dark:text-indigo-300">
          Admin · Operations
        </span>
      </div>
      <h1 class="text-2xl font-semibold tracking-tight text-slate-900 dark:text-white">
        Pipeline operations
      </h1>
      <p class="mt-1 max-w-2xl text-sm leading-6 text-slate-600 dark:text-slate-400">
        Review research workflow progress and token consumption across all accounts.
      </p>
    </div>

    <TokenUsagePanel scope="admin" />

    <div v-if="!pipelinesData?.pipelines?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No pipeline execution records found.
    </div>

    <div v-else class="divide-y divide-slate-100 overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm dark:divide-slate-800 dark:border-slate-800 dark:bg-slate-900">
      <div
        v-for="pipe in pipelinesData.pipelines"
        :key="pipe.workspace_id"
        class="flex flex-col justify-between gap-4 px-4 py-3.5 transition-colors hover:bg-slate-50 sm:flex-row sm:items-center dark:hover:bg-slate-800/60"
      >
        <div class="space-y-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold text-slate-900 dark:text-white">{{ pipe.company_name }}</span>
            <span class="text-xs text-slate-400">({{ pipe.role_title }})</span>
            <StatusBadge :status="pipe.status" size="xs" />
          </div>
          <div class="flex items-center gap-2 text-xs font-mono text-slate-500">
            <span>Phase: {{ pipe.current_phase }}</span>
            <span>•</span>
            <span>Progress: {{ pipe.progress_percent }}%</span>
          </div>
        </div>

        <NuxtLink
          :to="`/admin/pipelines/${pipe.workspace_id.replace(/^workspace:/, '').replace(/`/g, '')}`"
          class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium shrink-0"
        >
          Inspect Telemetry →
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
