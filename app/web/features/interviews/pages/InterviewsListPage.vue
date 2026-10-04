<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { filterWorkspaces } from '../utils/interview.util'
import WorkspaceCard from '../components/WorkspaceCard.vue'
import WorkspaceSearchFilter from '../components/WorkspaceSearchFilter.vue'

const route = useRoute()
const router = useRouter()
const { listWorkspaces } = useInterviewService()

const { data: workspaces, pending, error } = await useAsyncData(
  'interviews-list',
  () => listWorkspaces()
)

const searchQuery = ref<string>((route.query.search as string) || '')
const statusFilter = ref<string>((route.query.status as string) || 'all')

watch([searchQuery, statusFilter], () => {
  const query: Record<string, string> = {}
  if (searchQuery.value.trim()) {
    query.search = searchQuery.value.trim()
  }
  if (statusFilter.value !== 'all') {
    query.status = statusFilter.value
  }
  router.replace({ query })
})

const filteredWorkspaces = computed(() => {
  return filterWorkspaces(workspaces.value || [], searchQuery.value, statusFilter.value)
})
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-5">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interview Workspaces
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Catalog of targeted company preparation runs, research provenance, and active progress.
        </p>
      </div>

      <NuxtLink
        to="/interviews/new"
        class="inline-flex items-center px-4 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm self-start sm:self-auto"
      >
        + New Interview Preparation
      </NuxtLink>
    </div>

    <WorkspaceSearchFilter
      v-model:search-query="searchQuery"
      v-model:status-filter="statusFilter"
    />

    <div v-if="pending" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="i in 3"
        :key="i"
        class="h-44 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 animate-pulse p-6"
      ></div>
    </div>

    <div
      v-else-if="error"
      class="p-6 rounded-lg bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-400 text-xs"
    >
      Failed to load workspaces from backend.
    </div>

    <div
      v-else-if="filteredWorkspaces.length === 0"
      class="text-center py-16 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg"
    >
      <h3 class="text-sm font-semibold text-slate-900 dark:text-white">
        No matching interview workspaces
      </h3>
      <p class="text-xs text-slate-500 mt-1 mb-4">
        Try adjusting your filter or start a new preparation workspace.
      </p>
      <NuxtLink
        to="/interviews/new"
        class="inline-flex items-center px-3.5 py-1.5 text-xs font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition"
      >
        Create Interview Preparation
      </NuxtLink>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
      <WorkspaceCard
        v-for="ws in filteredWorkspaces"
        :key="ws.id"
        :workspace="ws"
      />
    </div>
  </div>
</template>
