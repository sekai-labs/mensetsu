<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  X,
  Wrench,
  ExternalLink,
  Loader2,
  FileText,
  DollarSign,
} from 'lucide-vue-next'
import type {
  WorkflowTask,
  AgentExecution,
  ToolInvocation,
  Delegation,
  Artifact,
  WorkflowEvent,
} from '~/models/generated/workflows.v1'
import { useWorkflowService } from '~/services/workflow.service'
import AgentStepFeed from '~/components/workflow/AgentStepFeed.vue'

const props = withDefaults(
  defineProps<{
    runId: string
    task: WorkflowTask | null
    execution: AgentExecution | null
    executionId?: string | null
    isExecutionLoading?: boolean
    isExecutionNotFound?: boolean
    delegations: Delegation[]
    artifacts: Artifact[]
    events?: WorkflowEvent[]
    isOpen: boolean
  }>(),
  {
    executionId: null,
    isExecutionLoading: false,
    isExecutionNotFound: false,
    events: () => [],
  }
)
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select-task', taskId: string): void
  (e: 'select-execution', executionId: string): void
  (e: 'select-artifact', artifactId: string): void
}>()

const workflowService = useWorkflowService()
const tools = ref<ToolInvocation[]>([])
const toolsCursor = ref<string | null>(null)
const isLoadingTools = ref(false)
const selectedTool = ref<ToolInvocation | null>(null)
const toolOutputContent = ref<string>('')
const toolOutputHasMore = ref(false)
const toolOutputOffset = ref(0)
const isLoadingToolOutput = ref(false)

const loadToolsForExecution = async (reset = true) => {
  if (!props.execution?.id) {
    tools.value = []
    return
  }
  if (reset) {
    toolsCursor.value = null
    tools.value = []
  }
  isLoadingTools.value = true
  try {
    const res = await workflowService.getAgentTools(props.runId, props.execution.id, {
      cursor: toolsCursor.value || undefined,
      limit: 10,
    })
    if (reset) {
      tools.value = res.items
    } else {
      tools.value = [...tools.value, ...res.items]
    }
    toolsCursor.value = res.next_cursor || null
  } catch (err) {
    console.warn('[ExecutionInspector] Failed to load tools:', err)
  } finally {
    isLoadingTools.value = false
  }
}

watch(
  () => props.execution?.id,
  () => {
    selectedTool.value = null
    toolOutputContent.value = ''
    loadToolsForExecution(true)
  },
  { immediate: true }
)

const selectTool = async (tool: ToolInvocation) => {
  selectedTool.value = tool
  toolOutputContent.value = tool.output_summary || ''
  toolOutputOffset.value = 0
  toolOutputHasMore.value = false

  isLoadingToolOutput.value = true
  try {
    const res = await workflowService.getToolOutput(props.runId, tool.id, {
      offset: 0,
      limit: 2000,
    })
    toolOutputContent.value = res.content
    toolOutputHasMore.value = res.has_more
    toolOutputOffset.value = res.offset + res.limit
  } catch (err) {
    console.warn('[ExecutionInspector] Failed to load tool output:', err)
  } finally {
    isLoadingToolOutput.value = false
  }
}

const loadMoreToolOutput = async () => {
  if (!selectedTool.value || !toolOutputHasMore.value || isLoadingToolOutput.value) return
  isLoadingToolOutput.value = true
  try {
    const res = await workflowService.getToolOutput(props.runId, selectedTool.value.id, {
      offset: toolOutputOffset.value,
      limit: 2000,
    })
    toolOutputContent.value += res.content
    toolOutputHasMore.value = res.has_more
    toolOutputOffset.value = res.offset + res.limit
  } catch (err) {
    console.warn('[ExecutionInspector] Failed to load more tool output:', err)
  } finally {
    isLoadingToolOutput.value = false
  }
}


const associatedArtifacts = computed(() => {
  if (!props.execution) return []
  return props.artifacts.filter(
    (a) => a.agent_execution_id === props.execution?.id || props.execution?.artifact_ids?.includes(a.id)
  )
})

const formatDuration = (ms?: number | null) => {
  if (!ms) return null
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

const formatJson = (val: unknown) => {
  if (typeof val === 'string') return val
  try {
    return JSON.stringify(val, null, 2)
  } catch {
    return String(val)
  }
}
</script>

<template>
  <aside
    v-if="isOpen"
    class="w-full lg:w-[450px] shrink-0 border-l border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 flex flex-col h-full overflow-hidden shadow-lg z-30"
  >

    <div class="p-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between gap-3 shrink-0 bg-slate-50/50 dark:bg-slate-900/50">
      <div class="min-w-0">
        <div class="text-[10px] font-mono uppercase tracking-wider text-slate-400">
          {{ (execution || isExecutionLoading || isExecutionNotFound || executionId) ? 'Agent Execution Inspector' : 'Task Inspector' }}
        </div>
        <div class="text-sm font-semibold text-slate-900 dark:text-white truncate">
          {{ execution?.agent_id || task?.name || executionId || 'Inspection' }}
        </div>
      </div>
      <button
        class="p-1 rounded text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
        @click="emit('close')"
      >
        <X class="w-4 h-4" />
      </button>
    </div>


    <div class="flex-1 overflow-y-auto p-4 space-y-6 text-sm">

      <section v-if="task" class="space-y-3">
        <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono">
          Task Overview
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-800/40 rounded-lg border border-slate-200 dark:border-slate-800 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Task ID</span>
            <span class="font-mono text-xs font-medium text-slate-700 dark:text-slate-300">{{ task.id }}</span>
          </div>
          <div v-if="task.task_type" class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Task Type</span>
            <span class="font-mono text-xs font-medium text-slate-700 dark:text-slate-300">{{ task.task_type }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Status</span>
            <span class="font-mono text-xs capitalize text-slate-700 dark:text-slate-300">{{ task.status }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Assigned Agent</span>
            <span class="font-mono text-xs text-indigo-600 dark:text-indigo-400">{{ task.assigned_agent }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Attempt</span>
            <span class="font-mono text-xs text-slate-700 dark:text-slate-300">{{ task.attempt || 1 }}</span>
          </div>
          <div v-if="task.error" class="p-2 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded text-xs text-rose-700 dark:text-rose-400 font-mono">
            {{ task.error }}
          </div>
        </div>
      </section>

      <section v-if="isExecutionNotFound" class="space-y-4">
        <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono">
          Agent Execution
        </div>
        <div class="p-6 bg-slate-50 dark:bg-slate-800/40 rounded-lg border border-slate-200 dark:border-slate-800 text-center space-y-2">
          <div class="text-xs font-semibold text-slate-700 dark:text-slate-300">Execution not found</div>
          <p class="text-xs text-slate-500 font-mono">{{ executionId }}</p>
        </div>
      </section>

      <section v-else-if="isExecutionLoading && !execution" class="space-y-4">
        <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono">
          Agent Execution
        </div>
        <div class="p-4 bg-slate-50 dark:bg-slate-800/40 rounded-lg border border-slate-200 dark:border-slate-800 space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500">Execution ID</span>
            <span class="font-mono text-xs text-slate-700 dark:text-slate-300 font-medium">{{ executionId }}</span>
          </div>
          <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-3/4"></div>
          <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-1/2"></div>
          <div class="h-12 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-full"></div>
        </div>
      </section>

      <section v-else-if="execution" class="space-y-4">
        <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono">
          Agent Execution
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-800/40 rounded-lg border border-slate-200 dark:border-slate-800 space-y-2 text-xs">
          <div class="flex items-center justify-between">
            <span class="text-slate-500">Execution ID</span>
            <span class="font-mono font-medium text-slate-700 dark:text-slate-300">{{ execution.id }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-slate-500">Agent Role</span>
            <span class="font-mono text-indigo-600 dark:text-indigo-400 font-medium">{{ execution.agent_id }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-slate-500">Status</span>
            <span class="font-mono capitalize font-medium text-slate-700 dark:text-slate-300">{{ execution.status }}</span>
          </div>
          <div v-if="execution.model_name" class="flex items-center justify-between">
            <span class="text-slate-500">Model</span>
            <span class="font-mono text-slate-700 dark:text-slate-300">{{ execution.model_name }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-slate-500">Delegation Depth</span>
            <span class="font-mono text-slate-700 dark:text-slate-300">{{ execution.delegation_depth }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-slate-500">Retries</span>
            <span class="font-mono text-slate-700 dark:text-slate-300">{{ execution.retries }}</span>
          </div>
          <div v-if="execution.parent_execution_id" class="flex items-center justify-between">
            <span class="text-slate-500">Parent Agent</span>
            <button
              class="font-mono text-indigo-600 dark:text-indigo-400 hover:underline flex items-center gap-1"
              @click="emit('select-execution', execution.parent_execution_id!)"
            >
              <span>{{ execution.parent_execution_id }}</span>
              <ExternalLink class="w-3 h-3" />
            </button>
          </div>
        </div>


        <div v-if="execution.objective" class="space-y-1">
          <div class="text-xs text-slate-500 font-medium">Objective</div>
          <div class="p-2.5 bg-slate-50 dark:bg-slate-800/40 rounded border border-slate-200 dark:border-slate-800 text-xs leading-relaxed text-slate-700 dark:text-slate-300">
            {{ execution.objective }}
          </div>
        </div>

        <div v-if="execution.public_summary" class="space-y-1">
          <div class="text-xs text-slate-500 font-medium">Summary</div>
          <div class="p-2.5 bg-slate-50 dark:bg-slate-800/40 rounded border border-slate-200 dark:border-slate-800 text-xs leading-relaxed text-slate-700 dark:text-slate-300">
            {{ execution.public_summary }}
          </div>
        </div>


        <div class="space-y-2">
          <div class="text-xs font-medium text-slate-500 flex items-center gap-1.5">
            <DollarSign class="w-3.5 h-3.5 text-slate-400" />
            <span>Budgets Used vs Limits</span>
          </div>
          <div class="grid grid-cols-2 gap-2 text-[11px] font-mono">
            <div class="p-2 bg-slate-50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-800 rounded">
              <span class="text-slate-500 block">Model Calls</span>
              <span class="font-semibold text-slate-900 dark:text-white">
                {{ execution.budgets_used?.model_calls || 0 }} / {{ execution.budgets?.model_calls || '∞' }}
              </span>
            </div>
            <div class="p-2 bg-slate-50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-800 rounded">
              <span class="text-slate-500 block">Tool Calls</span>
              <span class="font-semibold text-slate-900 dark:text-white">
                {{ execution.budgets_used?.tool_calls || 0 }} / {{ execution.budgets?.tool_calls || '∞' }}
              </span>
            </div>
            <div class="p-2 bg-slate-50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-800 rounded">
              <span class="text-slate-500 block">Tokens In</span>
              <span class="font-semibold text-slate-900 dark:text-white">
                {{ execution.budgets_used?.tokens_in || 0 }} / {{ execution.budgets?.tokens_in || '∞' }}
              </span>
            </div>
            <div class="p-2 bg-slate-50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-800 rounded">
              <span class="text-slate-500 block">Tokens Out</span>
              <span class="font-semibold text-slate-900 dark:text-white">
                {{ execution.budgets_used?.tokens_out || 0 }} / {{ execution.budgets?.tokens_out || '∞' }}
              </span>
            </div>
          </div>
        </div>

        <div class="space-y-2">
          <AgentStepFeed
            :execution-id="execution.id"
            :events="events"
            @select-execution="(id) => emit('select-execution', id)"
          />
        </div>

        <div class="space-y-2">
          <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono flex items-center justify-between">
            <div class="flex items-center gap-1.5">
              <Wrench class="w-3.5 h-3.5 text-slate-400" />
              <span>Tool Calls ({{ tools.length }})</span>
            </div>
            <span v-if="isLoadingTools" class="text-[10px] text-slate-400 flex items-center gap-1">
              <Loader2 class="w-3 h-3 animate-spin" />
              Loading...
            </span>
          </div>

          <div v-if="tools.length === 0 && !isLoadingTools" class="text-xs text-slate-500 italic p-3 bg-slate-50 dark:bg-slate-800/30 rounded border border-slate-200 dark:border-slate-800">
            No tool calls recorded for this agent execution.
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="tool in tools"
              :key="tool.id"
              class="border rounded-lg p-2.5 transition-colors cursor-pointer"
              :class="selectedTool?.id === tool.id ? 'border-indigo-500 bg-indigo-50/20 dark:bg-indigo-950/20' : 'border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50'"
              data-testid="tool-invocation-row"
              @click="selectTool(tool)"
            >
              <div class="flex items-center justify-between text-xs">
                <span class="font-mono font-semibold text-slate-900 dark:text-white">{{ tool.tool_name }}</span>
                <span class="font-mono text-[10px] px-1.5 py-0.5 rounded border capitalize" :class="tool.status === 'completed' ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-400 dark:border-emerald-800' : 'bg-slate-100 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700'">
                  {{ tool.status }}
                </span>
              </div>
              <div class="flex items-center gap-3 text-[10px] font-mono text-slate-500 mt-1">
                <span>{{ formatDuration(tool.duration_ms) || '0ms' }}</span>
                <span>{{ tool.started_at }}</span>
              </div>
            </div>

            <button
              v-if="toolsCursor"
              class="w-full py-1 text-xs text-indigo-600 dark:text-indigo-400 hover:underline flex items-center justify-center gap-1"
              :disabled="isLoadingTools"
              @click="loadToolsForExecution(false)"
            >
              <Loader2 v-if="isLoadingTools" class="w-3 h-3 animate-spin" />
              <span>Load More Tools</span>
            </button>
          </div>


          <div v-if="selectedTool" class="mt-4 p-3 border border-indigo-200 dark:border-indigo-900/60 rounded-lg bg-indigo-50/10 dark:bg-indigo-950/10 space-y-3">
            <div class="text-xs font-semibold text-slate-900 dark:text-white flex items-center justify-between">
              <span>Tool: {{ selectedTool.tool_name }}</span>
              <span class="text-[10px] font-mono text-slate-500">{{ selectedTool.id }}</span>
            </div>


            <div class="space-y-1">
              <span class="text-[10px] uppercase font-mono text-slate-500">Sanitized Input</span>
              <pre class="p-2 bg-slate-900 text-slate-100 rounded text-[11px] font-mono overflow-x-auto max-h-36 leading-relaxed">{{ formatJson(selectedTool.sanitized_input) }}</pre>
            </div>


            <div class="space-y-1">
              <div class="flex items-center justify-between">
                <span class="text-[10px] uppercase font-mono text-slate-500">Output</span>
                <button
                  v-if="toolOutputHasMore"
                  class="text-[10px] text-indigo-600 dark:text-indigo-400 hover:underline flex items-center gap-1"
                  :disabled="isLoadingToolOutput"
                  @click="loadMoreToolOutput"
                >
                  <Loader2 v-if="isLoadingToolOutput" class="w-2.5 h-2.5 animate-spin" />
                  <span>Load More Output</span>
                </button>
              </div>
              <pre class="p-2 bg-slate-900 text-slate-100 rounded text-[11px] font-mono overflow-x-auto max-h-48 leading-relaxed whitespace-pre-wrap">{{ toolOutputContent }}</pre>
            </div>

            <div v-if="selectedTool.error" class="p-2 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded text-xs text-rose-700 dark:text-rose-400 font-mono">
              {{ selectedTool.error }}
            </div>
          </div>
        </div>


        <div v-if="associatedArtifacts.length > 0" class="space-y-2">
          <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider font-mono flex items-center gap-1.5">
            <FileText class="w-3.5 h-3.5 text-slate-400" />
            <span>Artifacts ({{ associatedArtifacts.length }})</span>
          </div>
          <div class="space-y-1.5">
            <div
              v-for="art in associatedArtifacts"
              :key="art.id"
              class="p-2 border border-slate-200 dark:border-slate-800 rounded bg-slate-50/50 dark:bg-slate-800/40 flex items-center justify-between text-xs"
            >
              <div class="truncate min-w-0 mr-2">
                <div class="font-medium text-slate-900 dark:text-white truncate">{{ art.title }}</div>
                <div class="font-mono text-[10px] text-slate-500">{{ art.type }}</div>
              </div>
              <button
                class="text-indigo-600 dark:text-indigo-400 hover:underline font-mono text-[11px] shrink-0"
                @click="emit('select-artifact', art.id)"
              >
                Inspect
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>
  </aside>
</template>
