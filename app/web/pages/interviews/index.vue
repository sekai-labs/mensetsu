<script setup lang="ts">
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { filterWorkspaces } from '~/features/interviews/utils/interview.util'
import StatusBadge from '~/components/ui/StatusBadge.vue'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const { listWorkspaces } = useInterviewService()
const { data: workspaces, pending, error } = await useAsyncData('all-workspaces', () => listWorkspaces())

const searchQuery = ref('')
const statusFilter = ref('all')

const filteredWorkspaces = computed(() => {
  return filterWorkspaces(workspaces.value || [], searchQuery.value, statusFilter.value)
})
</script>

<template>
  <div class="space-y-6">

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-5">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('workspaces.title') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          {{ $t('workspaces.subtitle') }}
        </p>
      </div>

      <NuxtLink to="/interviews/new">
        <AppButton size="sm">
          {{ $t('workspaces.newWorkspace') }}
        </AppButton>
      </NuxtLink>
    </div>


    <div class="flex flex-col sm:flex-row gap-3 items-center justify-between">
      <div class="w-full sm:w-72">
        <AppInput
          v-model="searchQuery"
          :placeholder="$t('workspaces.searchPlaceholder')"
        />
      </div>

      <div class="flex items-center gap-1.5 self-start sm:self-auto text-xs">
        <button
          v-for="st in ['all', 'ready', 'processing', 'failed']"
          :key="st"
          type="button"
          class="px-2.5 py-1 rounded-md capitalize font-medium transition-colors"
          :class="statusFilter === st
            ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
            : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'"
          @click="statusFilter = st"
        >
          {{ st }}
        </button>
      </div>
    </div>


    <div v-if="pending" class="p-12 text-center text-xs text-slate-400">
      Loading workspaces...
    </div>

    <div v-else-if="error" class="p-4 rounded-md bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-xs text-rose-700 dark:text-rose-300">
      Failed to load workspaces from backend.
    </div>

    <div v-else-if="!filteredWorkspaces.length" class="p-12 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      {{ $t('workspaces.empty') }}
    </div>

    <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
      <div
        v-for="ws in filteredWorkspaces"
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
          <div v-if="ws.status === 'processing'" class="text-right">
            <div class="text-[11px] font-mono text-slate-400">Pipeline</div>
            <div class="text-xs font-mono font-semibold text-indigo-600 dark:text-indigo-400 tabular-nums">
              {{ ws.progress_percent }}%
            </div>
          </div>

          <NuxtLink :to="`/interviews/${ws.id}/overview`">
            <AppButton variant="secondary" size="xs">
              {{ $t('workspaces.openWorkspace') }}
            </AppButton>
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
