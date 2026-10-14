<script setup lang="ts">
import { useAdminService } from '~/features/admin/services/admin.service'
import StatusBadge from '~/components/ui/StatusBadge.vue'

definePageMeta({
  layout: 'default',
})

const { listPipelines } = useAdminService()
const { data: pipelinesData } = await useAsyncData('admin-pipelines', () => listPipelines())
</script>

<template>
  <div class="space-y-6">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        Durable Pipeline Executions
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
        Monitor asynchronous research workflows, Temporal activities, and step-level telemetry.
      </p>
    </div>

    <div v-if="!pipelinesData?.pipelines?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No pipeline execution records found.
    </div>

    <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
      <div
        v-for="pipe in pipelinesData.pipelines"
        :key="pipe.workspace_id"
        class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
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
