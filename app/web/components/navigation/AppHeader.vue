<script setup lang="ts">
import { computed } from 'vue'
import { Menu, Plus } from 'lucide-vue-next'
import type { WorkspaceResponse } from '~/features/interviews/models/interview.model'
import type { WorkspaceStatus, PipelinePhase } from '~/models/common.model'
import WorkspaceSwitcher from './WorkspaceSwitcher.vue'
import ResearchActivityIndicator from './ResearchActivityIndicator.vue'
import UserMenu from './UserMenu.vue'

interface Props {
  workspaceId?: string
  workspace?: WorkspaceResponse | null
  status?: WorkspaceStatus | string
  currentPhase?: PipelinePhase | string
  progressPercent?: number
  isConnected?: boolean
  message?: string
  isMobileMenuOpen?: boolean
  activeRunId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  workspaceId: '',
  workspace: null,
  status: 'ready',
  currentPhase: '',
  progressPercent: 0,
  isConnected: false,
  message: '',
  isMobileMenuOpen: false,
})

const emit = defineEmits<{
  (e: 'toggle-mobile-menu'): void
}>()

const hasWorkspaceContext = computed(() => {
  return Boolean(props.workspaceId)
})
</script>

<template>
  <header data-testid="app-header" class="border-b border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 sticky top-0 z-40">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-13 flex items-center justify-between gap-3">

      <div class="flex items-center gap-2 sm:gap-3 min-w-0">

        <button
          v-if="hasWorkspaceContext"
          type="button"
          class="lg:hidden p-1.5 rounded-md text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-indigo-500 shrink-0"
          :aria-expanded="isMobileMenuOpen"
          aria-label="Open Workspace Navigation"
          @click="emit('toggle-mobile-menu')"
        >
          <Menu class="w-5 h-5" />
        </button>


        <NuxtLink to="/dashboard" class="flex items-center gap-2 shrink-0 group py-1" aria-label="Mensetsu Dashboard">
          <div class="w-6 h-6 rounded bg-indigo-600 flex items-center justify-center text-white font-bold text-xs tracking-wider group-hover:bg-indigo-500 transition-colors shadow-sm">
            面
          </div>
          <span class="font-semibold text-slate-900 dark:text-white tracking-tight text-sm hidden sm:inline">
            Mensetsu
          </span>
        </NuxtLink>


        <template v-if="hasWorkspaceContext">
          <span class="text-slate-300 dark:text-slate-700 hidden sm:inline">/</span>
          <WorkspaceSwitcher
            :current-workspace-id="workspaceId"
            :current-workspace="workspace"
          />
        </template>

        <NuxtLink
          to="/workflows"
          class="hidden md:inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-md text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors ml-1"
          active-class="text-indigo-600 dark:text-indigo-400 font-semibold bg-indigo-50 dark:bg-indigo-950/50"
        >
          <span>Workflows</span>
        </NuxtLink>
      </div>


      <div class="flex items-center gap-2 sm:gap-3 shrink-0">
        <NuxtLink
          v-if="activeRunId"
          :to="`/workflows/runs/${activeRunId}`"
          class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-mono font-medium rounded-md bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-400 border border-indigo-200 dark:border-indigo-800 hover:bg-indigo-100 dark:hover:bg-indigo-900/60 transition-colors"
        >
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          <span>Active Run</span>
        </NuxtLink>

        <ResearchActivityIndicator
          v-if="hasWorkspaceContext"
          :workspace-id="workspaceId"
          :status="status"
          :current-phase="currentPhase"
          :progress-percent="progressPercent"
          :is-connected="isConnected"
          :message="message"
        />

        <NuxtLink
          to="/interviews/new"
          class="hidden sm:inline-flex items-center gap-1 px-2.5 py-1 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition-colors shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          aria-label="Create New Workspace"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>New</span>
        </NuxtLink>


        <UserMenu />
      </div>
    </div>
  </header>
</template>
