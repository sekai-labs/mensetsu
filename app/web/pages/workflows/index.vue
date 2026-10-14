<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  Workflow,
  Plus,
  Search,
  Filter,
  ArrowRight,
  Sparkles,
  AlertCircle,
  CheckCircle2,
  Ban,
  CircleDot,
  Loader2,
  ChevronLeft,
  ChevronRight,
} from 'lucide-vue-next'
import { useWorkflowService } from '~/services/workflow.service'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type {
  WorkflowType,
  RunStatus,
} from '~/models/generated/workflows.v1'
import type { CreateWorkflowRunPayload } from '~/services/workflow.service'
import CreateWorkflowRunDialog from '~/components/workflow/CreateWorkflowRunDialog.vue'

definePageMeta({
  layout: 'default',
})

const router = useRouter()
const { listWorkflowRuns, createWorkflowRun } = useWorkflowService()
const { listWorkspaces } = useInterviewService()

const filterType = ref<WorkflowType | 'all'>('all')
const filterStatus = ref<RunStatus | 'all'>('all')
const searchQuery = ref('')
const currentCursor = ref<string | undefined>(undefined)
const cursorHistory = ref<string[]>([])

const {
  data: runsPage,
  pending,
  error,
} = await useAsyncData(
  'workflow-runs',
  () =>
    listWorkflowRuns({
      type: filterType.value === 'all' ? undefined : filterType.value,
      status: filterStatus.value === 'all' ? undefined : filterStatus.value,
      cursor: currentCursor.value,
      limit: 20,
    }),
  {
    watch: [filterType, filterStatus, currentCursor],
  }
)

const { data: workspaces } = await useAsyncData('workspace-options', () => listWorkspaces(), {
  default: () => [],
})

const isCreateDialogOpen = ref(false)


const runs = computed(() => runsPage.value?.items || [])
const nextCursor = computed(() => runsPage.value?.next_cursor)

const filteredRuns = computed(() => {
  return runs.value.filter((run) => {
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      const matchName = run.workflow_name ? run.workflow_name.toLowerCase().includes(q) : false
      const matchId = run.id.toLowerCase() === q || run.id.toLowerCase().includes(q)
      const matchPhase = run.current_phase ? run.current_phase.toLowerCase().includes(q) : false
      if (!matchName && !matchId && !matchPhase) return false
    }
    return true
  })
})

const handleCreateRun = async (payload: CreateWorkflowRunPayload) => {
  try {
    const newRun = await createWorkflowRun(payload)
    isCreateDialogOpen.value = false
    router.push(`/workflows/runs/${newRun.id}`)
  } catch (err) {
    console.error('Failed to create run:', err)
  }
}

const getStatusConfig = (status: RunStatus) => {
  switch (status) {
    case 'completed':
      return {
        badgeClass: 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-400 dark:border-emerald-800',
        icon: CheckCircle2,
      }
    case 'running':
      return {
        badgeClass: 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/60 dark:text-indigo-400 dark:border-indigo-800',
        icon: Loader2,
        iconClass: 'animate-spin',
      }
    case 'failed':
      return {
        badgeClass: 'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/60 dark:text-rose-400 dark:border-rose-800',
        icon: AlertCircle,
      }
    case 'cancelled':
      return {
        badgeClass: 'bg-slate-100 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700',
        icon: Ban,
      }
    case 'pending':
    default:
      return {
        badgeClass: 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-900 dark:text-slate-400 dark:border-slate-800',
        icon: CircleDot,
      }
  }
}

const formatDuration = (ms?: number | null) => {
  if (!ms) return '-'
  if (ms < 1000) return `${ms}ms`
  const sec = Math.floor(ms / 1000)
  if (sec < 60) return `${sec}s`
  const min = Math.floor(sec / 60)
  return `${min}m ${sec % 60}s`
}

const handleNextPage = () => {
  if (!nextCursor.value) return
  if (currentCursor.value) {
    cursorHistory.value.push(currentCursor.value)
  }
  currentCursor.value = nextCursor.value
}

const handlePrevPage = () => {
  const prev = cursorHistory.value.pop()
  currentCursor.value = prev
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 dark:text-white flex items-center gap-2">
          <Workflow class="w-6 h-6 text-indigo-600 dark:text-indigo-400" />
          <span>Workflow Execution Studio</span>
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Monitor deterministic pipelines and dynamic multi-agent orchestrations.
        </p>
      </div>

      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg text-xs font-semibold shadow-xs transition-colors self-start sm:self-auto"
        @click="isCreateDialogOpen = true"
      >
        <Plus class="w-4 h-4" />
        <span>New Workflow Run</span>
      </button>
    </div>


    <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-3 flex flex-wrap items-center justify-between gap-3 shadow-xs">
      <div class="flex flex-wrap items-center gap-3">

        <div class="flex items-center gap-1.5 text-xs text-slate-500">
          <Filter class="w-3.5 h-3.5" />
          <select
            v-model="filterStatus"
            class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-2.5 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono"
          >
            <option value="all">All Statuses</option>
            <option value="running">Running</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="cancelled">Cancelled</option>
            <option value="pending">Pending</option>
          </select>
        </div>


        <div class="flex items-center gap-1.5 text-xs text-slate-500">
          <select
            v-model="filterType"
            class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-2.5 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono"
          >
            <option value="all">All Types</option>
            <option value="static">Static Pipelines</option>
            <option value="dynamic">Dynamic Autonomous</option>
          </select>
        </div>
      </div>


      <div class="relative w-full sm:w-64">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter by name, ID, phase..."
          class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded pl-8 pr-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono placeholder:text-slate-400"
        />
        <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-2" />
      </div>
    </div>


    <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden shadow-xs">
      <div v-if="pending" class="p-12 text-center text-xs text-slate-400 flex items-center justify-center gap-2 font-mono">
        <Loader2 class="w-4 h-4 animate-spin text-indigo-500" />
        <span>Loading workflow runs...</span>
      </div>

      <div v-else-if="error" class="p-8 text-center text-xs text-rose-500 font-mono">
        Failed to load workflow runs: {{ error.message }}
      </div>

      <div v-else-if="filteredRuns.length === 0" class="p-12 text-center space-y-3">
        <Workflow class="w-8 h-8 text-slate-400 mx-auto" />
        <div class="text-xs text-slate-500 font-medium">No workflow runs found.</div>
        <p class="text-xs text-slate-400 max-w-sm mx-auto">
          Start a new workflow run or adjust active filters above to inspect agent activity.
        </p>
      </div>

      <div v-else class="overflow-x-auto">
        <table class="w-full text-left border-collapse text-xs">
          <thead>
            <tr class="border-b border-slate-200 dark:border-slate-800 bg-slate-50/75 dark:bg-slate-800/50 text-[11px] font-mono uppercase text-slate-500 sticky top-0">
              <th class="py-2.5 px-4 font-semibold">Workflow / ID</th>
              <th class="py-2.5 px-4 font-semibold">Type</th>
              <th class="py-2.5 px-4 font-semibold">Status</th>
              <th class="py-2.5 px-4 font-semibold">Progress / Phase</th>
              <th class="py-2.5 px-4 font-semibold">Tasks (Done/Total)</th>
              <th class="py-2.5 px-4 font-semibold">Started</th>
              <th class="py-2.5 px-4 font-semibold">Duration</th>
              <th class="py-2.5 px-4 font-semibold text-right">Action</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100 dark:divide-slate-800/60 font-mono text-xs">
            <tr
              v-for="run in filteredRuns"
              :key="run.id"
              class="hover:bg-slate-50/50 dark:hover:bg-slate-800/50 transition-colors cursor-pointer"
              @click="router.push(`/workflows/runs/${run.id}`)"
            >

              <td class="py-3 px-4">
                <div class="font-sans font-semibold text-slate-900 dark:text-white">{{ run.workflow_name }}</div>
                <div class="text-[10px] text-slate-400 truncate max-w-[180px]">{{ run.id }}</div>
              </td>


              <td class="py-3 px-4">
                <span
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] border"
                  :class="run.type === 'dynamic' ? 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/60 dark:text-amber-400 dark:border-amber-800' : 'bg-slate-50 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700'"
                >
                  <Sparkles v-if="run.type === 'dynamic'" class="w-2.5 h-2.5 text-amber-500" />
                  <span class="capitalize">{{ run.type }}</span>
                </span>
              </td>


              <td class="py-3 px-4">
                <span
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium border"
                  :class="getStatusConfig(run.status).badgeClass"
                >
                  <component
                    :is="getStatusConfig(run.status).icon"
                    class="w-3 h-3 shrink-0"
                    :class="getStatusConfig(run.status).iconClass"
                  />
                  <span class="capitalize">{{ run.status }}</span>
                </span>
              </td>


              <td class="py-3 px-4">
                <div class="flex items-center gap-2">

                  <span v-if="run.progress?.percentage !== null && run.progress?.percentage !== undefined" class="font-semibold text-slate-800 dark:text-slate-200">
                    {{ run.progress.percentage }}%
                  </span>
                  <span v-else class="text-[10px] text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/40 px-1 py-0.2 rounded border border-amber-200 dark:border-amber-800">
                    Estimate
                  </span>
                  <span class="text-slate-400 text-[11px] truncate max-w-[120px]">
                    ({{ run.current_phase || 'init' }})
                  </span>
                </div>
              </td>


              <td class="py-3 px-4 text-slate-600 dark:text-slate-300">
                <span>{{ run.progress?.completed || 0 }}</span>
                <span class="text-slate-400"> / </span>
                <span>{{ run.progress?.total_tasks || 0 }}</span>
                <span v-if="run.progress?.failed && run.progress.failed > 0" class="ml-1 text-rose-500 text-[10px]">
                  ({{ run.progress.failed }} fail)
                </span>
              </td>


              <td class="py-3 px-4 text-slate-500 text-[11px]">
                {{ run.started_at ? run.started_at.slice(0, 19).replace('T', ' ') : '-' }}
              </td>


              <td class="py-3 px-4 text-slate-500 text-[11px]">
                {{ formatDuration(run.duration_ms) }}
              </td>


              <td class="py-3 px-4 text-right">
                <button
                  class="p-1 rounded text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-slate-100 dark:hover:bg-slate-800"
                  @click.stop="router.push(`/workflows/runs/${run.id}`)"
                >
                  <ArrowRight class="w-4 h-4" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>


      <div class="p-3 bg-slate-50/50 dark:bg-slate-800/30 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs font-mono text-slate-500">
        <div>
          Showing {{ filteredRuns.length }} runs
        </div>
        <div class="flex items-center gap-2">
          <button
            class="px-2 py-1 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-50 disabled:opacity-40 flex items-center gap-1"
            :disabled="cursorHistory.length === 0"
            @click="handlePrevPage"
          >
            <ChevronLeft class="w-3.5 h-3.5" />
            <span>Previous</span>
          </button>
          <button
            class="px-2 py-1 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-50 disabled:opacity-40 flex items-center gap-1"
            :disabled="!nextCursor"
            @click="handleNextPage"
          >
            <span>Next</span>
            <ChevronRight class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>


    <CreateWorkflowRunDialog
      :is-open="isCreateDialogOpen"
      :workspaces="workspaces || []"
      @close="isCreateDialogOpen = false"
      @create="handleCreateRun"
    />
  </div>
</template>
