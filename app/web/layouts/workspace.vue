<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { useInterviewEventsService } from '~/features/interviews/services/interview-events.service'
import { useWorkflowService } from '~/services/workflow.service'
import AppHeader from '~/components/navigation/AppHeader.vue'
import WorkspaceSidebar from '~/components/navigation/WorkspaceSidebar.vue'
import MobileNavigationDrawer from '~/components/navigation/MobileNavigationDrawer.vue'
import type { WorkspaceResponse } from '~/features/interviews/models/interview.model'
const route = useRoute()
const isMobileMenuOpen = ref(false)

const workspaceId = computed(() => String(route.params.id || ''))

const { getWorkspace } = useInterviewService()
const { listWorkflowRuns } = useWorkflowService()

const { data: workspace, refresh: refreshWorkspace } = await useAsyncData<WorkspaceResponse | null>(
  () => `ws-shell-${workspaceId.value}`,
  () => (workspaceId.value ? getWorkspace(workspaceId.value) : Promise.resolve(null)),
  { watch: [workspaceId] }
)

const { data: runsData } = await useAsyncData(
  () => `ws-runs-${workspaceId.value}`,
  () => (workspaceId.value ? listWorkflowRuns({ workspace_id: workspaceId.value, limit: 1 }) : Promise.resolve(null)),
  { watch: [workspaceId] }
)

const activeRunId = computed(() => {
  const latest = runsData.value?.items?.[0]
  return latest ? latest.id : null
})

const { currentPhase, progressPercent, status: sseStatus, isConnected, message } =
  useInterviewEventsService(workspaceId.value, () => {
    refreshWorkspace()
  })

const effectiveStatus = computed(() => sseStatus.value || workspace.value?.status || 'pending')
const effectivePercent = computed(() => progressPercent.value || workspace.value?.progress_percent || 0)
const effectivePhase = computed(() => currentPhase.value || workspace.value?.current_phase || 'preparing_input')
</script>

<template>
  <div class="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 font-sans selection:bg-indigo-600 selection:text-white">

    <AppHeader
      :workspace-id="workspaceId"
      :workspace="workspace"
      :status="effectiveStatus"
      :current-phase="effectivePhase"
      :progress-percent="effectivePercent"
      :is-connected="isConnected"
      :message="message"
      :is-mobile-menu-open="isMobileMenuOpen"
      :active-run-id="activeRunId"
      @toggle-mobile-menu="isMobileMenuOpen = !isMobileMenuOpen"
    />


    <MobileNavigationDrawer
      :is-open="isMobileMenuOpen"
      :workspace-id="workspaceId"
      :workspace="workspace"
      :status="effectiveStatus"
      @close="isMobileMenuOpen = false"
    />


    <div class="flex-1 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-6 flex flex-col lg:flex-row gap-8">

      <div class="hidden lg:block shrink-0">
        <WorkspaceSidebar
          :workspace-id="workspaceId"
          :workspace="workspace"
          :status="effectiveStatus"
        />
      </div>


      <main class="flex-1 min-w-0" id="main-content">
        <slot />
      </main>
    </div>


    <footer class="border-t border-slate-200 dark:border-slate-800 py-4 text-center text-xs text-slate-400 dark:text-slate-500">
      Mensetsu Workspace — Verified Evidence & Deep Technical Dossier
    </footer>
  </div>
</template>
