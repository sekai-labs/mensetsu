<script setup lang="ts">
import { computed } from 'vue'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { calculateWorkspaceStatistics } from '~/features/interviews/utils/interview.util'
import WorkspaceCard from '~/features/interviews/components/WorkspaceCard.vue'

const { listWorkspaces } = useInterviewService()
const { data: workspaces, pending, error } = await useAsyncData(
  'dashboard-workspaces',
  () => listWorkspaces()
)

const stats = computed(() => calculateWorkspaceStatistics(workspaces.value || []))
const recentWorkspaces = computed(() => (workspaces.value || []).slice(0, 6))
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-6">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Preparation Workspace Overview
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Autonomous research status, technical curriculum generation, and active interview preparation runs.
        </p>
      </div>

      <NuxtLink
        to="/interviews/new"
        class="inline-flex items-center px-4 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm self-start sm:self-auto"
      >
        + Launch New Preparation
      </NuxtLink>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-5">
      <div class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Active Research Jobs</span>
        <div class="text-2xl font-bold font-mono text-indigo-600 dark:text-indigo-400 mt-1">
          {{ stats.active }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Pipelines parsing CVs and crawling tech stacks</p>
      </div>

      <div class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Ready Workspaces</span>
        <div class="text-2xl font-bold font-mono text-emerald-600 dark:text-emerald-400 mt-1">
          {{ stats.ready }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Curricula & rubrics ready for drills</p>
      </div>

      <div class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Total Workspaces</span>
        <div class="text-2xl font-bold font-mono text-slate-900 dark:text-white mt-1">
          {{ stats.total }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Historical company interview preps</p>
      </div>
    </div>

    <div v-if="stats.failed > 0" class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-xs flex items-center justify-between">
      <span class="text-rose-700 dark:text-rose-400 font-medium">
        {{ stats.failed }} workspace research job(s) encountered an error or failed validation.
      </span>
      <NuxtLink to="/interviews" class="font-semibold text-rose-800 dark:text-rose-300 hover:underline">
        Review Failed Jobs →
      </NuxtLink>
    </div>

    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Recent Interview Workspaces</h2>
        <NuxtLink to="/interviews" class="text-xs font-semibold text-indigo-600 hover:text-indigo-500 transition">
          View all →
        </NuxtLink>
      </div>

      <div v-if="pending" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <div v-for="i in 3" :key="i" class="h-40 rounded-lg bg-slate-100 dark:bg-slate-800 animate-pulse"></div>
      </div>

      <div v-else-if="error" class="p-6 rounded-lg bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 text-xs text-rose-700 dark:text-rose-400">
        Failed to fetch recent workspaces from API. Ensure Core API backend is running.
      </div>

      <div v-else-if="recentWorkspaces.length === 0" class="p-12 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg">
        No active workspaces. Launch a new preparation run to begin.
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <WorkspaceCard
          v-for="ws in recentWorkspaces"
          :key="ws.id"
          :workspace="ws"
        />
      </div>
    </div>
  </div>
</template>
