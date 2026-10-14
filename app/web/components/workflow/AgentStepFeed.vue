<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  Activity,
  Wrench,
  CheckCircle2,
  XCircle,
  Loader2,
  Bookmark,
  GitFork,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Pause,
  Play,
} from 'lucide-vue-next'
import type { WorkflowEvent } from '~/models/generated/workflows.v1'

export interface AgentStepItem {
  id: string
  seq: number
  ts: string
  kind: 'activity' | 'tool' | 'evidence' | 'finding' | 'delegation'
  activityText?: string
  toolName?: string
  toolArg?: string
  toolDurationMs?: number
  toolStatus?: 'running' | 'completed' | 'failed'
  toolSummary?: string
  toolError?: string
  toolInvocationId?: string
  evidenceTitle?: string
  evidenceUrl?: string
  evidenceSourceId?: string
  findingClaim?: string
  findingCategory?: string
  findingConfidence?: number
  delegationId?: string
  childExecutionId?: string
  delegationSummary?: string
}

const props = withDefaults(
  defineProps<{
    executionId?: string | null
    events?: WorkflowEvent[]
    compact?: boolean
  }>(),
  {
    executionId: null,
    events: () => [],
    compact: false,
  }
)

const emit = defineEmits<{
  (e: 'select-execution', executionId: string): void
  (e: 'open-citation', sourceId: string): void
}>()

const isAutoScrollPaused = ref(false)
const containerRef = ref<HTMLElement | null>(null)
const expandedSteps = ref<Record<string, boolean>>({})

const sanitizeToolArg = (input: unknown): string => {
  if (!input) return ''
  if (typeof input === 'string') return input
  if (typeof input === 'object' && input !== null) {
    const obj = input as Record<string, unknown>
    if (typeof obj.query === 'string') return obj.query
    if (typeof obj.url === 'string') return obj.url
    if (typeof obj.statement === 'string') return obj.statement
    if (typeof obj.objective === 'string') return obj.objective
    if (typeof obj.title === 'string') return obj.title
    try {
      const serialized = JSON.stringify(input)
      return serialized.length > 80 ? `${serialized.slice(0, 77)}...` : serialized
    } catch {
      return ''
    }
  }
  return String(input)
}

const formatDuration = (ms?: number): string => {
  if (ms === undefined || ms === null) return ''
  if (ms < 1000) return `${Math.round(ms)}ms`
  const s = (ms / 1000).toFixed(1)
  return `${s}s`
}

const formatTimestamp = (ts?: string): string => {
  if (!ts) return ''
  return ts.length >= 19 ? ts.slice(11, 19) : ts
}

const steps = computed<AgentStepItem[]>(() => {
  const filteredEvents = (props.events || []).filter((ev) => {
    if (!props.executionId) return true
    return ev.execution_id === props.executionId
  })

  const sortedEvents = [...filteredEvents].sort((a, b) => a.seq - b.seq)

  const toolMap = new Map<string, AgentStepItem>()
  const items: AgentStepItem[] = []

  for (const ev of sortedEvents) {
    const type = ev.type
    const payload = ((ev.payload || {}) as unknown) as Record<string, unknown>
    const ts = ev.ts || ''
    const seq = ev.seq
    if (type === 'agent.activity') {
      const description = String(payload.description || payload.activity_type || '')
      items.push({
        id: `act_${ev.event_id || seq}`,
        seq,
        ts,
        kind: 'activity',
        activityText: description,
      })
    } else if (type === 'tool.started') {
      const invocationId = String(ev.tool_invocation_id || payload.tool_invocation_id || `tool_${seq}`)
      const toolName = String(payload.tool_name || '')
      const toolArg = sanitizeToolArg(payload.sanitized_input)

      const toolStep: AgentStepItem = {
        id: `tool_${invocationId}`,
        seq,
        ts,
        kind: 'tool',
        toolName,
        toolArg,
        toolStatus: 'running',
        toolInvocationId: invocationId,
      }
      toolMap.set(invocationId, toolStep)
      items.push(toolStep)
    } else if (type === 'tool.completed') {
      const invocationId = String(ev.tool_invocation_id || payload.tool_invocation_id || '')
      const existing = invocationId ? toolMap.get(invocationId) : undefined
      const durationMs = typeof payload.duration_ms === 'number' ? payload.duration_ms : undefined
      const summary = typeof payload.summary === 'string' ? payload.summary : undefined

      if (existing) {
        existing.toolStatus = 'completed'
        existing.toolDurationMs = durationMs
        existing.toolSummary = summary
      } else {
        items.push({
          id: `tool_c_${ev.event_id || seq}`,
          seq,
          ts,
          kind: 'tool',
          toolName: String(payload.tool_name || 'tool'),
          toolStatus: 'completed',
          toolDurationMs: durationMs,
          toolSummary: summary,
          toolInvocationId: invocationId || undefined,
        })
      }
    } else if (type === 'tool.failed') {
      const invocationId = String(ev.tool_invocation_id || payload.tool_invocation_id || '')
      const existing = invocationId ? toolMap.get(invocationId) : undefined
      const error = typeof payload.error === 'string' ? payload.error : 'Tool failed'

      if (existing) {
        existing.toolStatus = 'failed'
        existing.toolError = error
      } else {
        items.push({
          id: `tool_f_${ev.event_id || seq}`,
          seq,
          ts,
          kind: 'tool',
          toolName: String(payload.tool_name || 'tool'),
          toolStatus: 'failed',
          toolError: error,
          toolInvocationId: invocationId || undefined,
        })
      }
    } else if (type === 'evidence.saved') {
      items.push({
        id: `ev_${ev.event_id || seq}`,
        seq,
        ts,
        kind: 'evidence',
        evidenceTitle: String(payload.title || 'Evidence saved'),
        evidenceUrl: typeof payload.url === 'string' ? payload.url : undefined,
        evidenceSourceId: typeof payload.source_id === 'string' ? payload.source_id : undefined,
      })
    } else if (type === 'finding.saved') {
      items.push({
        id: `find_${ev.event_id || seq}`,
        seq,
        ts,
        kind: 'finding',
        findingClaim: String(payload.claim || ''),
        findingCategory: String(payload.category || 'Finding'),
        findingConfidence: typeof payload.confidence === 'number' ? payload.confidence : undefined,
      })
    } else if (type === 'delegation.started') {
      const childExecutionId = typeof payload.child_execution_id === 'string' ? payload.child_execution_id : undefined
      const delegationId = typeof payload.delegation_id === 'string' ? payload.delegation_id : undefined
      items.push({
        id: `del_${ev.event_id || seq}`,
        seq,
        ts,
        kind: 'delegation',
        delegationId,
        childExecutionId,
      })
    } else if (type === 'delegation.completed') {
      const delegationId = typeof payload.delegation_id === 'string' ? payload.delegation_id : undefined
      const childExecutionId = typeof payload.child_execution_id === 'string' ? payload.child_execution_id : undefined
      const summary = typeof payload.summary === 'string' ? payload.summary : undefined

      const existingDel = delegationId ? items.find((i) => i.delegationId === delegationId) : undefined
      if (existingDel) {
        existingDel.delegationSummary = summary
      } else {
        items.push({
          id: `del_c_${ev.event_id || seq}`,
          seq,
          ts,
          kind: 'delegation',
          delegationId,
          childExecutionId,
          delegationSummary: summary,
        })
      }
    }
  }

  return items.sort((a, b) => a.seq - b.seq)
})

const toggleExpanded = (stepId: string) => {
  expandedSteps.value[stepId] = !expandedSteps.value[stepId]
}

const togglePause = () => {
  isAutoScrollPaused.value = !isAutoScrollPaused.value
}

watch(
  () => steps.value.length,
  async () => {
    if (isAutoScrollPaused.value) return
    await nextTick()
    if (containerRef.value) {
      containerRef.value.scrollTop = containerRef.value.scrollHeight
    }
  }
)
</script>

<template>
  <div
    class="flex flex-col bg-slate-900 border border-slate-800 rounded-lg overflow-hidden text-xs font-mono shadow-sm"
    :class="compact ? 'max-h-72' : 'h-full'"
    data-testid="agent-step-feed"
  >
    <div class="h-9 px-3 bg-slate-950 border-b border-slate-800 flex items-center justify-between shrink-0 select-none">
      <div class="flex items-center gap-2 text-slate-400 font-semibold uppercase text-[10px] tracking-wider">
        <Activity class="w-3.5 h-3.5 text-indigo-400" />
        <span>Agent Activity Feed</span>
        <span class="text-slate-500 font-normal">({{ steps.length }})</span>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] transition-colors"
          :class="isAutoScrollPaused ? 'bg-amber-950/60 text-amber-400 border border-amber-800' : 'bg-slate-800 text-slate-400 hover:text-slate-200'"
          :title="isAutoScrollPaused ? 'Resume auto-scroll' : 'Pause auto-scroll'"
          @click="togglePause"
        >
          <Pause v-if="!isAutoScrollPaused" class="w-3 h-3" />
          <Play v-else class="w-3 h-3" />
          <span>{{ isAutoScrollPaused ? 'Paused' : 'Auto-scroll' }}</span>
        </button>
      </div>
    </div>

    <div
      ref="containerRef"
      class="flex-1 overflow-y-auto divide-y divide-slate-800/60 p-2 space-y-1"
      :class="compact ? 'max-h-60' : ''"
    >
      <div v-if="steps.length === 0" class="p-6 text-center text-slate-500 text-xs">
        No agent step activity recorded yet.
      </div>

      <div
        v-for="step in steps"
        :key="step.id"
        class="pt-1.5 pb-1.5 px-2 rounded hover:bg-slate-800/40 transition-colors leading-relaxed"
        :data-testid="`step-row-${step.kind}`"
      >
        <div v-if="step.kind === 'activity'" class="flex items-start gap-2 text-slate-300">
          <span class="text-slate-500 text-[10px] shrink-0 pt-0.5 w-14">{{ formatTimestamp(step.ts) }}</span>
          <Activity class="w-3.5 h-3.5 text-indigo-400 shrink-0 mt-0.5" />
          <span class="flex-1 break-words text-slate-300">{{ step.activityText }}</span>
        </div>

        <div v-else-if="step.kind === 'tool'" class="space-y-1">
          <div class="flex items-start gap-2">
            <span class="text-slate-500 text-[10px] shrink-0 pt-0.5 w-14">{{ formatTimestamp(step.ts) }}</span>

            <Wrench class="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />

            <div class="flex-1 min-w-0 flex flex-wrap items-center gap-1.5">
              <span class="font-semibold text-amber-300">{{ step.toolName }}</span>
              <span v-if="step.toolArg" class="text-slate-400 truncate max-w-xs text-[11px]">
                "{{ step.toolArg }}"
              </span>
            </div>

            <div class="flex items-center gap-1.5 shrink-0">
              <span v-if="step.toolDurationMs !== undefined" class="text-[10px] text-slate-400">
                {{ formatDuration(step.toolDurationMs) }}
              </span>

              <span v-if="step.toolStatus === 'running'" class="flex items-center gap-1 text-[10px] text-amber-400">
                <Loader2 class="w-3 h-3 animate-spin" />
              </span>
              <CheckCircle2
                v-else-if="step.toolStatus === 'completed'"
                class="w-3.5 h-3.5 text-emerald-400"
              />
              <XCircle
                v-else-if="step.toolStatus === 'failed'"
                class="w-3.5 h-3.5 text-rose-400"
              />

              <button
                v-if="step.toolSummary || step.toolError"
                type="button"
                class="p-0.5 text-slate-500 hover:text-slate-300"
                :title="expandedSteps[step.id] ? 'Collapse result' : 'Expand result'"
                @click="toggleExpanded(step.id)"
              >
                <ChevronDown v-if="expandedSteps[step.id]" class="w-3 h-3" />
                <ChevronRight v-else class="w-3 h-3" />
              </button>
            </div>
          </div>

          <div
            v-if="expandedSteps[step.id] && (step.toolSummary || step.toolError)"
            class="ml-16 mr-2 p-2 bg-slate-950 rounded border border-slate-800 text-[11px] text-slate-300 whitespace-pre-wrap break-all"
            data-testid="tool-result-summary"
          >
            <div v-if="step.toolError" class="text-rose-400 font-semibold mb-1">
              Error: {{ step.toolError }}
            </div>
            <div v-if="step.toolSummary" class="text-slate-300">
              {{ step.toolSummary }}
            </div>
          </div>
        </div>

        <div v-else-if="step.kind === 'evidence'" class="flex items-start gap-2 text-emerald-300">
          <span class="text-slate-500 text-[10px] shrink-0 pt-0.5 w-14">{{ formatTimestamp(step.ts) }}</span>
          <Bookmark class="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
          <div class="flex-1 min-w-0 flex flex-wrap items-center gap-2">
            <span class="font-medium text-emerald-300">{{ step.evidenceTitle }}</span>
            <a
              v-if="step.evidenceUrl"
              :href="step.evidenceUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-1 text-[11px] text-indigo-400 hover:underline break-all"
            >
              <span>{{ step.evidenceUrl }}</span>
              <ExternalLink class="w-3 h-3 shrink-0" />
            </a>
            <button
              v-else-if="step.evidenceSourceId"
              type="button"
              class="inline-flex items-center gap-1 text-[11px] text-indigo-400 hover:underline"
              @click="emit('open-citation', step.evidenceSourceId!)"
            >
              <span>Source #{{ step.evidenceSourceId }}</span>
            </button>
          </div>
        </div>

        <div v-else-if="step.kind === 'finding'" class="flex items-start gap-2 text-indigo-300">
          <span class="text-slate-500 text-[10px] shrink-0 pt-0.5 w-14">{{ formatTimestamp(step.ts) }}</span>
          <Bookmark class="w-3.5 h-3.5 text-indigo-400 shrink-0 mt-0.5" />
          <div class="flex-1 min-w-0 flex flex-wrap items-center gap-1.5">
            <span class="px-1.5 py-0.2 rounded bg-indigo-950/60 border border-indigo-800 text-[10px] text-indigo-400 uppercase">
              {{ step.findingCategory }}
            </span>
            <span class="text-slate-200">{{ step.findingClaim }}</span>
            <span v-if="step.findingConfidence !== undefined" class="text-[10px] text-emerald-400">
              ({{ Math.round(step.findingConfidence * 100) }}%)
            </span>
          </div>
        </div>

        <div v-else-if="step.kind === 'delegation'" class="flex items-start gap-2 text-indigo-300">
          <span class="text-slate-500 text-[10px] shrink-0 pt-0.5 w-14">{{ formatTimestamp(step.ts) }}</span>
          <GitFork class="w-3.5 h-3.5 text-indigo-400 shrink-0 mt-0.5" />
          <div class="flex-1 min-w-0 flex flex-wrap items-center gap-1.5">
            <span class="text-slate-400">Delegated to execution:</span>
            <button
              v-if="step.childExecutionId"
              type="button"
              class="font-semibold text-indigo-400 hover:underline flex items-center gap-1"
              @click="emit('select-execution', step.childExecutionId!)"
            >
              <span>{{ step.childExecutionId }}</span>
              <ExternalLink class="w-3 h-3" />
            </button>
            <span v-else class="text-slate-500">{{ step.delegationId }}</span>
            <span v-if="step.delegationSummary" class="text-slate-400 text-[11px] block w-full mt-0.5">
              {{ step.delegationSummary }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
