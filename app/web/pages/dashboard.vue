<script setup lang="ts">
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { calculateWorkspaceStatistics } from '~/features/interviews/utils/interview.util'
import { useAuthService } from '~/features/auth/services/auth.service'
import StatusBadge from '~/components/ui/StatusBadge.vue'
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'default',
})

const { user } = useAuthService()
const { listWorkspaces } = useInterviewService()
const { data: workspaces, pending, error } = await useAsyncData('dashboard-workspaces', () => listWorkspaces())

const stats = computed(() => {
  return calculateWorkspaceStatistics(workspaces.value || [])
})

const recentWorkspaces = computed(() => {
  return (workspaces.value || []).slice(0, 5)
})
</script>

<template>
  <div class="space-y-8">

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-5">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('dashboard.title') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          {{ $t('dashboard.welcome', { name: user?.name || user?.email || 'Engineer' }) }}
        </p>
      </div>

      <NuxtLink to="/interviews/new">
        <AppButton size="sm">
          {{ $t('dashboard.newWorkspace') }}
        </AppButton>
      </NuxtLink>
    </div>


    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('dashboard.activeWorkspaces') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-slate-900 dark:text-white mt-1">
          {{ stats.total }}
        </div>
      </div>
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('dashboard.completedWorkspaces') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-emerald-600 dark:text-emerald-400 mt-1">
          {{ stats.ready }}
        </div>
      </div>
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('dashboard.analyzing') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-indigo-600 dark:text-indigo-400 mt-1">
          {{ stats.active }}
        </div>
      </div>
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('dashboard.attentionNeeded') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-rose-600 dark:text-rose-400 mt-1">
          {{ stats.failed }}
        </div>
      </div>
    </div>


    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
          {{ $t('dashboard.activePreparations') }}
        </h2>
        <NuxtLink to="/interviews" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium">
          {{ $t('dashboard.viewAll', { count: stats.total }) }}
        </NuxtLink>
      </div>

      <div v-if="pending" class="p-8 text-center text-xs text-slate-400">
        Loading workspaces...
      </div>

      <div v-else-if="error" class="p-4 rounded-md bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-xs text-rose-700 dark:text-rose-300">
        Could not retrieve workspaces. Ensure the backend service is running.
      </div>

      <div v-else-if="!recentWorkspaces.length" class="p-10 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-center space-y-3">
        <p class="text-xs text-slate-500 dark:text-slate-400">
          {{ $t('dashboard.empty') }}
        </p>
        <NuxtLink to="/interviews/new">
          <AppButton size="sm">
            {{ $t('dashboard.createFirst') }}
          </AppButton>
        </NuxtLink>
      </div>

      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="ws in recentWorkspaces"
          :key="ws.id"
          class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <div class="space-y-1 min-w-0">
            <div class="flex items-center gap-2.5">
              <NuxtLink
                :to="`/interviews/${ws.id}/overview`"
                class="text-sm font-semibold text-slate-900 dark:text-white hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors truncate"
              >
                {{ ws.company_name }}
              </NuxtLink>
              <StatusBadge :status="ws.status" size="xs" />
            </div>
            <div class="flex items-center gap-2 text-xs text-slate-500">
              <span class="font-medium text-slate-700 dark:text-slate-300">{{ ws.role_title }}</span>
              <span>•</span>
              <span class="capitalize">{{ ws.seniority || 'Mid' }}</span>
              <span>•</span>
              <span class="font-mono text-[11px]">{{ new Date(ws.created_at).toLocaleDateString() }}</span>
            </div>
          </div>

          <div class="flex items-center gap-3 shrink-0">
            <NuxtLink :to="`/interviews/${ws.id}/overview`">
              <AppButton variant="secondary" size="xs">
                {{ $t('dashboard.openWorkspace') }}
              </AppButton>
            </NuxtLink>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
