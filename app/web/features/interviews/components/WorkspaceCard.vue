<script setup lang="ts">
import StatusBadge from '~/components/ui/StatusBadge.vue'
import { formatDate } from '~/utils/formatters'
import type { WorkspaceResponse } from '../models/interview.model'

defineProps<{
  workspace: WorkspaceResponse
  showViewButton?: boolean
}>()
</script>

<template>
  <div
    class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm hover:border-slate-300 dark:hover:border-slate-700 transition flex flex-col justify-between gap-4"
  >
    <div class="space-y-3">
      <div class="flex items-start justify-between gap-3">
        <div class="space-y-0.5">
          <NuxtLink
            :to="`/interviews/${workspace.id}/overview`"
            class="font-semibold text-sm text-slate-900 dark:text-white hover:text-indigo-600 dark:hover:text-indigo-400 transition"
          >
            {{ workspace.company_name }}
          </NuxtLink>
          <div class="text-xs text-slate-500 line-clamp-1">
            {{ workspace.role_title }}
          </div>
        </div>
        <StatusBadge :status="workspace.status" size="sm" />
      </div>

      <div class="space-y-1.5">
        <div class="flex items-center justify-between text-xs font-mono text-slate-500">
          <span class="text-[11px] uppercase tracking-wider text-slate-400">Pipeline Progress</span>
          <span class="font-semibold text-slate-700 dark:text-slate-300">{{ workspace.progress_percent }}%</span>
        </div>
        <div class="w-full bg-slate-100 dark:bg-slate-800 rounded-full h-1.5 overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-300"
            :class="{
              'bg-emerald-500': workspace.status === 'ready',
              'bg-rose-500': workspace.status === 'failed',
              'bg-indigo-600 animate-pulse': workspace.status === 'processing',
              'bg-amber-500': workspace.status === 'pending'
            }"
            :style="{ width: `${workspace.progress_percent}%` }"
          ></div>
        </div>
      </div>
    </div>

    <div class="pt-3 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs text-slate-400">
      <span>{{ formatDate(workspace.created_at) }}</span>
      <NuxtLink
        :to="`/interviews/${workspace.id}/overview`"
        class="inline-flex items-center gap-1 font-semibold text-indigo-600 dark:text-indigo-400 hover:text-indigo-500 transition"
      >
        <span>Open</span>
        <span>→</span>
      </NuxtLink>
    </div>
  </div>
</template>
