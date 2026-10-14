<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { VueFlow, useVueFlow, EdgeLabelRenderer, getSmoothStepPath } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import dagre from '@dagrejs/dagre'
import {
  Workflow,
  Clock,
  Activity,
  Terminal,
  FileText,
  ZoomIn,
  ZoomOut,
  Maximize2,
  GitFork,
  ArrowLeft,
  AlertCircle,
  Loader2,
} from 'lucide-vue-next'
import { useWorkflowRunStore } from '~/composables/useWorkflowStore'
import { useWorkflowService } from '~/services/workflow.service'
import TaskNode from '~/components/workflow/TaskNode.vue'
import DelegationNode from '~/components/workflow/DelegationNode.vue'
import ExecutionInspector from '~/components/workflow/ExecutionInspector.vue'
import ActivityTimeline from '~/components/workflow/ActivityTimeline.vue'
import ExecutionLogsViewer from '~/components/workflow/ExecutionLogsViewer.vue'
import ArtifactsViewer from '~/components/workflow/ArtifactsViewer.vue'
import type {
  WorkflowTask,
} from '~/models/generated/workflows.v1'

definePageMeta({
  layout: 'default',
})

const route = useRoute()
const router = useRouter()
const runId = computed(() => String(route.params.id))

const { store } = useWorkflowRunStore(runId)
const workflowService = useWorkflowService()
const activeBottomTab = ref<'activity' | 'logs' | 'artifacts'>('activity')
const isInspectorOpen = ref(false)
const selectedEdgeInfo = ref<Record<string, unknown> | null>(null)
const isMounted = ref(false)
watch(
  () => route.query,
  (q) => {
    if (q.tab && (q.tab === 'activity' || q.tab === 'logs' || q.tab === 'artifacts')) {
      activeBottomTab.value = q.tab
    }
    if (q.task && store.value) {
      store.value.selectedTaskId.value = String(q.task)
      isInspectorOpen.value = true
    }
    if (q.exec && store.value) {
      store.value.selectedAgentId.value = String(q.exec)
      isInspectorOpen.value = true
    }
  },
  { immediate: true }
)

const updateQueryParams = () => {
  const query: Record<string, string> = {}
  if (activeBottomTab.value) query.tab = activeBottomTab.value
  if (store.value?.selectedTaskId.value) query.task = store.value.selectedTaskId.value
  if (store.value?.selectedAgentId.value) query.exec = store.value.selectedAgentId.value
  router.replace({ query })
}


const { fitView, zoomIn, zoomOut } = useVueFlow()

const layoutDagre = (nodes: any[], edges: any[], direction = 'TB', nodeWidth = 260, nodeHeight = 110, nodesep = 50, ranksep = 70) => {
  const dagreGraph = new dagre.graphlib.Graph()
  dagreGraph.setDefaultEdgeLabel(() => ({}))
  dagreGraph.setGraph({ rankdir: direction, nodesep, ranksep })

  for (const node of nodes) {
    dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight })
  }

  for (const edge of edges) {
    dagreGraph.setEdge(edge.source, edge.target)
  }

  dagre.layout(dagreGraph)

  const halfW = nodeWidth / 2
  const halfH = nodeHeight / 2

  return nodes.map((node) => {
    const nodeWithPos = dagreGraph.node(node.id)
    return {
      ...node,
      position: {
        x: nodeWithPos ? nodeWithPos.x - halfW : 0,
        y: nodeWithPos ? nodeWithPos.y - halfH : 0,
      },
    }
  })
}


const taskFlowElements = computed(() => {
  if (!store.value) return { nodes: [], edges: [] }
  const rawNodes = store.value.taskGraph.value.nodes.map((n) => ({
    id: n.id,
    type: 'task',
    data: {
      ...n.data,
      id: n.id,
      name: n.label,
    },
    position: { x: 0, y: 0 },
  }))

  const rawEdges = store.value.taskGraph.value.edges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    animated: false,
    style: { stroke: '#94a3b8', strokeWidth: 2 },
    data: { kind: e.kind },
  }))

  const nodes = layoutDagre(rawNodes, rawEdges)
  return { nodes, edges: rawEdges }
})


const delegationFlowElements = computed(() => {
  if (!store.value) return { nodes: [], edges: [] }
  const rawNodes = store.value.delegationGraph.value.nodes.map((n) => ({
    id: n.id,
    type: 'delegation',
    data: {
      ...n.data,
      id: n.id,
    },
    label: n.label,
    position: { x: 0, y: 0 },
  }))

  const rawEdges = store.value.delegationGraph.value.edges.map((e) => {
    const status = e.data?.status || 'pending'
    const labelText = e.data?.objective ? `${status}: ${e.data.objective}` : status
    return {
      id: e.id || `del_edge_${e.source}_${e.target}`,
      source: e.source,
      target: e.target,
      type: 'smoothstep',
      animated: status === 'running',
      class: 'stroke-indigo-400 dark:stroke-indigo-400 stroke-2',
      interactionWidth: 24,
      label: labelText,
      labelStyle: { fill: '#4f46e5', fontWeight: 600, fontSize: 11, fontFamily: 'monospace' },
      labelBgStyle: { fill: '#ffffff', fillOpacity: 0.95 },
      labelBgPadding: [6, 4] as [number, number],
      labelBgBorderRadius: 4,
      data: e.data,
    }
  })
  const nodes = layoutDagre(rawNodes, rawEdges, 'LR', 256, 210, 70, 80)
  return { nodes, edges: rawEdges }
})

const currentNodes = computed(() => {
  if (store.value?.graphMode.value === 'delegations') {
    return delegationFlowElements.value.nodes
  }
  return taskFlowElements.value.nodes
})

const currentEdges = computed(() => {
  if (store.value?.graphMode.value === 'delegations') {
    return delegationFlowElements.value.edges
  }
  return taskFlowElements.value.edges
})


const onNodeClick = (event: any) => {
  const node = event.node
  if (!store.value) return
  if (store.value.graphMode.value === 'delegations') {
    store.value.selectedAgentId.value = node.id
    store.value.selectedTaskId.value = node.data?.task_id || null
  } else {
    store.value.selectedTaskId.value = node.id
    const exec = store.value.agents.value.find((a) => a.task_id === node.id)
    store.value.selectedAgentId.value = exec ? exec.id : null
  }
  selectedEdgeInfo.value = null
  isInspectorOpen.value = true
  updateQueryParams()
}

const onEdgeClick = (event: any) => {
  const edge = event.edge
  selectedEdgeInfo.value = {
    id: edge.id,
    source: edge.source,
    target: edge.target,
    kind: edge.data?.kind || 'dependency',
    ...edge.data,
  }
}

const selectedTask = computed(() => {
  const taskId = store.value?.selectedTaskId.value
  if (!taskId || !store.value) return null
  const node = store.value.taskGraph.value.nodes.find((n) => n.id === taskId)
  if (!node) return null
  return {
    id: node.id,
    run_id: runId.value,
    task_type: node.data.task_type || undefined,
    name: node.label,
    assigned_agent: node.data.assigned_agent,
    status: node.data.status,
    attempt: node.data.attempt,
    required: node.data.required,
    added_in_revision: node.data.added_in_revision,
  } as WorkflowTask
})

const isExecutionLoading = ref(false)
const isExecutionNotFound = ref(false)

const selectedExecution = computed(() => {
  const execId = store.value?.selectedAgentId.value
  if (!execId || !store.value) return null
  return store.value.agents.value.find((a) => a.id === execId) || null
})

watch(
  () => store.value?.selectedAgentId.value,
  async (execId) => {
    if (!execId || !store.value) {
      isExecutionLoading.value = false
      isExecutionNotFound.value = false
      return
    }
    const existing = store.value.agents.value.find((a) => a.id === execId)
    if (existing) {
      isExecutionLoading.value = false
      isExecutionNotFound.value = false
      return
    }
    isExecutionLoading.value = true
    isExecutionNotFound.value = false
    try {
      const freshAgent = await workflowService.getAgentExecution(runId.value, execId)
      if (!store.value.agents.value.some((a) => a.id === freshAgent.id)) {
        store.value.agents.value.push(freshAgent)
      }
    } catch (err: any) {
      if (err?.statusCode === 404 || err?.status === 404) {
        isExecutionNotFound.value = true
      }
    } finally {
      isExecutionLoading.value = false
    }
  },
  { immediate: true }
)

const onSelectTask = (taskId: string) => {
  if (store.value) {
    store.value.selectedTaskId.value = taskId
    isInspectorOpen.value = true
    updateQueryParams()
  }
}

const onSelectExecution = (execId: string) => {
  if (store.value) {
    store.value.selectedAgentId.value = execId
    isInspectorOpen.value = true
    updateQueryParams()
  }
}

const onSelectArtifact = (_artId: string) => {
  activeBottomTab.value = 'artifacts'
  updateQueryParams()
}

onMounted(async () => {
  isMounted.value = true
  if (store.value) {
    await store.value.fetchFullSnapshot()
    store.value.connectSse()
    nextTick(() => {
      fitView()
    })
  }
})

onUnmounted(() => {
  if (store.value) {
    store.value.disconnectSse()
  }
})

const run = computed(() => store.value?.run.value)
const isConnected = computed(() => store.value?.isConnected.value)

const formatDuration = (ms?: number | null) => {
  if (!ms) return '-'
  if (ms < 1000) return `${ms}ms`
  const sec = Math.floor(ms / 1000)
  if (sec < 60) return `${sec}s`
  const min = Math.floor(sec / 60)
  return `${min}m ${sec % 60}s`
}
</script>

<template>
  <div class="h-[calc(100vh-64px)] flex flex-col bg-slate-50 dark:bg-slate-950 overflow-hidden">
    <header data-testid="run-header" class="h-14 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 px-4 flex items-center justify-between shrink-0 shadow-xs">
      <div class="flex items-center gap-3 min-w-0">
        <button
          class="p-1.5 rounded text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Back to workflows"
          @click="router.push('/workflows')"
        >
          <ArrowLeft class="w-4 h-4" />
        </button>

        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h1 class="text-xs sm:text-sm font-bold text-slate-900 dark:text-white truncate">
              {{ run?.workflow_name || 'Workflow Run' }}
            </h1>
            <span
              v-if="run"
              class="px-1.5 py-0.2 rounded text-[10px] font-mono border"
              :class="run.type === 'dynamic' ? 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/60 dark:text-amber-400 dark:border-amber-800' : 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700'"
            >
              {{ run.type }}
            </span>
          </div>
          <div class="text-[10px] font-mono text-slate-400 truncate">
            {{ runId }}
          </div>
        </div>
      </div>


      <div class="flex items-center gap-3 sm:gap-6 text-xs font-mono">
        <ClientOnly>
          <div class="flex items-center gap-1.5">
            <span
              class="w-2 h-2 rounded-full"
              :class="isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'"
            />
            <span class="text-[11px] text-slate-500 hidden sm:inline">
              {{ isConnected ? 'Live SSE' : 'Disconnected' }}
            </span>
          </div>

          <div v-if="run?.progress" class="flex items-center gap-2">
            <span class="text-slate-400 hidden sm:inline">Progress:</span>
            <span v-if="run.progress.percentage !== null && run.progress.percentage !== undefined" class="font-bold text-slate-800 dark:text-slate-200">
              {{ run.progress.percentage }}%
            </span>
            <span v-else class="text-[10px] text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/40 px-1 py-0.2 rounded border border-amber-200 dark:border-amber-800">
              Estimate
            </span>
            <span class="text-slate-500 text-[11px]">
              ({{ run.progress.completed }}/{{ run.progress.total_tasks }})
            </span>
          </div>

          <div v-if="run?.current_phase" class="hidden md:flex items-center gap-1.5">
            <span class="text-slate-400">Phase:</span>
            <span class="font-semibold text-slate-700 dark:text-slate-300 capitalize">{{ run.current_phase }}</span>
          </div>

          <div class="hidden sm:flex items-center gap-1.5 text-slate-500">
            <Clock class="w-3.5 h-3.5" />
            <span>{{ formatDuration(run?.duration_ms) }}</span>
          </div>
        </ClientOnly>
      </div>
    </header>


    <div class="flex-1 flex overflow-hidden relative">

      <div class="flex-1 flex flex-col min-w-0 overflow-hidden">

        <div class="h-10 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 px-4 flex items-center justify-between shrink-0 z-10">

          <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-800 p-0.5 rounded-lg text-xs font-medium">
            <button
              class="px-3 py-1 rounded transition-colors flex items-center gap-1.5"
              :class="store?.graphMode.value === 'tasks' ? 'bg-white dark:bg-slate-900 text-slate-900 dark:text-white shadow-xs' : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
              @click="store!.graphMode.value = 'tasks'"
            >
              <Workflow class="w-3.5 h-3.5" />
              <span>Task Graph</span>
            </button>
            <button
              class="px-3 py-1 rounded transition-colors flex items-center gap-1.5"
              :class="store?.graphMode.value === 'delegations' ? 'bg-white dark:bg-slate-900 text-slate-900 dark:text-white shadow-xs' : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
              @click="store!.graphMode.value = 'delegations'"
            >
              <GitFork class="w-3.5 h-3.5" />
              <span>Delegation Graph</span>
            </button>
          </div>


          <div class="flex items-center gap-1">
            <button
              class="p-1.5 rounded text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
              title="Zoom in"
              @click="() => zoomIn()"
            >
              <ZoomIn class="w-4 h-4" />
            </button>
            <button
              class="p-1.5 rounded text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
              title="Zoom out"
              @click="() => zoomOut()"
            >
              <ZoomOut class="w-4 h-4" />
            </button>
            <button
              class="p-1.5 rounded text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
              title="Fit graph"
              @click="() => fitView()"
            >
              <Maximize2 class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div class="flex-1 relative bg-slate-50 dark:bg-slate-950">
          <div v-if="store?.sectionErrors.value.run" class="absolute inset-0 flex items-center justify-center p-8 z-20 bg-white dark:bg-slate-950">
            <div class="max-w-md w-full p-6 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm text-center space-y-3">
              <div class="w-10 h-10 rounded-full bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 flex items-center justify-center mx-auto text-rose-600 dark:text-rose-400">
                <AlertCircle class="w-5 h-5" />
              </div>
              <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Run Not Accessible</h3>
              <p class="text-xs text-slate-500 leading-relaxed">
                {{ store.sectionErrors.value.run }}
              </p>
              <NuxtLink
                to="/workflows"
                class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded text-xs font-medium shadow-xs transition-colors"
              >
                <ArrowLeft class="w-3.5 h-3.5" />
                <span>Back to Workflows</span>
              </NuxtLink>
            </div>
          </div>
          <div v-else-if="!isMounted || !store || (!store.isInitialized.value && store.isLoading.value)" class="absolute inset-0 flex items-center justify-center p-8 z-10">
            <div class="w-full max-w-md p-6 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-xs space-y-3">
              <div class="h-4 bg-slate-200 dark:bg-slate-800 rounded animate-pulse w-3/4"></div>
              <div class="h-3 bg-slate-100 dark:bg-slate-800/60 rounded animate-pulse w-1/2"></div>
              <div class="grid grid-cols-2 gap-3 pt-2">
                <div class="h-16 bg-slate-100 dark:bg-slate-800/40 rounded animate-pulse"></div>
                <div class="h-16 bg-slate-100 dark:bg-slate-800/40 rounded animate-pulse"></div>
              </div>
            </div>
          </div>

          <div v-if="store?.isRefreshing.value" class="absolute top-3 right-3 z-20 flex items-center gap-1.5 px-2.5 py-1 bg-white/90 dark:bg-slate-900/90 border border-slate-200 dark:border-slate-800 rounded-md shadow-xs text-[11px] font-mono text-slate-500 pointer-events-none">
            <Loader2 class="w-3 h-3 animate-spin text-indigo-500" />
            <span>Refreshing...</span>
          </div>
          <VueFlow
            :nodes="currentNodes"
            :edges="currentEdges"
            :fit-view-on-init="true"
            class="h-full w-full"
            @node-click="onNodeClick"
            @edge-click="onEdgeClick"
          >
            <template #node-task="props">
              <TaskNode
                :id="props.id"
                :data="props.data"
                :selected="store?.selectedTaskId.value === props.id"
              />
            </template>

            <template #node-delegation="props">
              <DelegationNode
                :id="props.id"
                :label="String(props.label || '')"
                :data="props.data"
                :selected="store?.selectedAgentId.value === props.id"
              />
            </template>

            <template #edge-smoothstep="edgeProps">
              <g class="cursor-pointer" @click="onEdgeClick({ edge: edgeProps })">
                <path
                  :id="edgeProps.id"
                  class="vue-flow__edge-path stroke-indigo-400 dark:stroke-indigo-400 stroke-2"
                  :d="getSmoothStepPath(edgeProps)[0]"
                  :style="edgeProps.style"
                />
                <path
                  class="vue-flow__edge-interaction"
                  :d="getSmoothStepPath(edgeProps)[0]"
                  stroke-opacity="0"
                  stroke-width="24"
                  fill="none"
                />
                <EdgeLabelRenderer>
                  <div
                    :style="{
                      position: 'absolute',
                      transform: `translate(-50%, -50%) translate(${getSmoothStepPath(edgeProps)[1]}px,${getSmoothStepPath(edgeProps)[2]}px)`,
                      pointerEvents: 'all',
                    }"
                    data-testid="delegation-edge-label"
                    class="px-2 py-0.5 rounded border border-indigo-200 dark:border-indigo-800 bg-white dark:bg-slate-900 shadow-xs text-[11px] font-mono text-indigo-600 dark:text-indigo-400 font-semibold cursor-pointer select-none hover:bg-indigo-50 dark:hover:bg-slate-800 transition-colors"
                    @click.stop="onEdgeClick({ edge: edgeProps })"
                  >
                    {{ edgeProps.label || 'Delegation' }}
                  </div>
                </EdgeLabelRenderer>
              </g>
            </template>
          </VueFlow>


          <div
            v-if="selectedEdgeInfo"
            class="absolute top-4 left-4 p-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-lg text-xs font-mono z-20 space-y-1.5 max-w-sm"
          >
            <div class="flex items-center justify-between text-slate-500 font-bold uppercase text-[10px]">
              <span>Edge Dependency</span>
              <button class="hover:text-slate-800 dark:hover:text-white" @click="selectedEdgeInfo = null">✕</button>
            </div>
            <div><span class="text-slate-400">Source:</span> {{ selectedEdgeInfo.source }}</div>
            <div><span class="text-slate-400">Target:</span> {{ selectedEdgeInfo.target }}</div>
            <div><span class="text-slate-400">Kind:</span> {{ selectedEdgeInfo.kind }}</div>
            <div v-if="selectedEdgeInfo.objective" class="text-slate-600 dark:text-slate-300">
              <span class="text-slate-400">Objective:</span> {{ selectedEdgeInfo.objective }}
            </div>
          </div>
        </div>


        <div class="h-64 sm:h-72 border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 flex flex-col shrink-0">

          <div class="h-10 border-b border-slate-200 dark:border-slate-800 px-4 flex items-center justify-between shrink-0 bg-slate-50/50 dark:bg-slate-900/50">
            <div class="flex items-center gap-4 text-xs font-medium">
              <button
                class="py-2 border-b-2 flex items-center gap-1.5 transition-colors"
                :class="activeBottomTab === 'activity' ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400' : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
                @click="activeBottomTab = 'activity'; updateQueryParams()"
              >
                <Activity class="w-3.5 h-3.5" />
                <span>Activity Timeline</span>
                <ClientOnly>
                  <span class="text-[11px] text-slate-400">({{ store?.events.value.length || 0 }})</span>
                </ClientOnly>
              </button>

              <button
                class="py-2 border-b-2 flex items-center gap-1.5 transition-colors"
                :class="activeBottomTab === 'logs' ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400' : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
                @click="activeBottomTab = 'logs'; updateQueryParams()"
              >
                <Terminal class="w-3.5 h-3.5" />
                <span>Execution Logs</span>
                <ClientOnly>
                  <span class="text-[11px] text-slate-400">({{ store?.logs.value.length || 0 }})</span>
                </ClientOnly>
              </button>

              <button
                class="py-2 border-b-2 flex items-center gap-1.5 transition-colors"
                :class="activeBottomTab === 'artifacts' ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400' : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
                @click="activeBottomTab = 'artifacts'; updateQueryParams()"
              >
                <FileText class="w-3.5 h-3.5" />
                <span>Artifacts</span>
                <ClientOnly>
                  <span class="text-[11px] text-slate-400">({{ store?.artifacts.value.length || 0 }})</span>
                </ClientOnly>
              </button>
            </div>
          </div>


          <div class="flex-1 overflow-y-auto p-3">
            <ActivityTimeline
              v-if="activeBottomTab === 'activity'"
              :events="store?.events.value || []"
              @select-task="onSelectTask"
              @select-execution="onSelectExecution"
              @select-artifact="onSelectArtifact"
            />

            <ExecutionLogsViewer
              v-else-if="activeBottomTab === 'logs'"
              :logs="store?.logs.value || []"
              :is-paused="false"
            />

            <ArtifactsViewer
              v-else-if="activeBottomTab === 'artifacts'"
              :artifacts="store?.artifacts.value || []"
              @select-task="onSelectTask"
              @select-execution="onSelectExecution"
            />
          </div>
        </div>
      </div>


      <ExecutionInspector
        :run-id="runId"
        :task="selectedTask"
        :execution="selectedExecution"
        :execution-id="store?.selectedAgentId.value"
        :is-execution-loading="isExecutionLoading"
        :is-execution-not-found="isExecutionNotFound"
        :delegations="store?.delegations.value || []"
        :artifacts="store?.artifacts.value || []"
        :events="store?.events.value || []"
        :is-open="isInspectorOpen"
        @close="isInspectorOpen = false"
        @select-task="onSelectTask"
        @select-execution="onSelectExecution"
        @select-artifact="onSelectArtifact"
      />
    </div>
  </div>
</template>
