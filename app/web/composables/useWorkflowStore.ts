import { ref, shallowRef, computed, onMounted, onUnmounted, getCurrentInstance } from 'vue'
import type { Ref, ShallowRef } from 'vue'
import type {
  WorkflowRun,
  AgentExecution,
  Delegation,
  Artifact,
  LogEntry,
  TaskGraphResponse,
  DelegationGraphResponse,
  WorkflowEvent,
  TaskStatus,
  WorkflowCompletedPayload,
  WorkflowFailedPayload,
  WorkflowCancelledPayload,
  TaskStartedPayload,
  TaskCompletedPayload,
  TaskFailedPayload,
  TaskAddedPayload,
  AgentStartedPayload,
  AgentCompletedPayload,
  AgentFailedPayload,
} from '~/models/generated/workflows.v1'
import { WORKFLOW_EVENT_TYPES } from '~/models/generated/workflows.v1'
import {
  parseWorkflowEvent,
  isTerminalEventType,
} from '~/features/workflows/api/contract'
import { useWorkflowService } from '~/services/workflow.service'

export interface SectionErrors {
  run: string | null
  taskGraph: string | null
  delegationGraph: string | null
  agents: string | null
  delegations: string | null
  artifacts: string | null
  history: string | null
  timeline: string | null
}

export interface WorkflowRunInstance {
  run: Ref<WorkflowRun | null>
  taskGraph: Ref<TaskGraphResponse>
  delegationGraph: Ref<DelegationGraphResponse>
  agents: Ref<AgentExecution[]>
  delegations: Ref<Delegation[]>
  artifacts: Ref<Artifact[]>
  logs: Ref<LogEntry[]>
  events: Ref<WorkflowEvent[]>
  lastSeq: Ref<number>
  isConnected: Ref<boolean>
  connectionError: Ref<string | null>
  isLoading: Ref<boolean>
  isInitialized: Ref<boolean>
  isRefreshing: Ref<boolean>
  sectionErrors: Ref<SectionErrors>
  selectedTaskId: Ref<string | null>
  selectedAgentId: Ref<string | null>
  activeTab: Ref<'activity' | 'logs' | 'artifacts'>
  graphMode: Ref<'tasks' | 'delegations'>
  service: unknown
  fetchFullSnapshot: () => Promise<void>
  loadEventsHistory: (reset?: boolean) => Promise<void>
  loadOlderEvents: () => Promise<void>
  loadMoreLogs: () => Promise<void>
  connectSse: (eventSourceFactory?: (url: string) => EventSource) => void
  disconnectSse: () => void
  processIncomingEvent: (ev: WorkflowEvent) => void
  patchTaskStatus: (taskId: string, status: TaskStatus, error?: string | null) => void
}

function createRunStoreInstance(runId: string): WorkflowRunInstance {
  const service = useWorkflowService()

  const run = ref<WorkflowRun | null>(null)
  const taskGraph = ref<TaskGraphResponse>({ nodes: [], edges: [] })
  const delegationGraph = ref<DelegationGraphResponse>({ nodes: [], edges: [] })
  const agents = ref<AgentExecution[]>([])
  const delegations = ref<Delegation[]>([])
  const artifacts = ref<Artifact[]>([])
  const logs = ref<LogEntry[]>([])
  const events = ref<WorkflowEvent[]>([])
  const lastSeq = ref<number>(0)
  const isConnected = ref<boolean>(false)
  const connectionError = ref<string | null>(null)
  const isLoading = ref<boolean>(false)
  const isInitialized = ref<boolean>(false)
  const isRefreshing = ref<boolean>(false)
  const sectionErrors = ref<SectionErrors>({
    run: null,
    taskGraph: null,
    delegationGraph: null,
    agents: null,
    delegations: null,
    artifacts: null,
    history: null,
    timeline: null,
  })

  const selectedTaskId = ref<string | null>(null)
  const selectedAgentId = ref<string | null>(null)
  const activeTab = ref<'activity' | 'logs' | 'artifacts'>('activity')
  const graphMode = ref<'tasks' | 'delegations'>('tasks')

  const seenSeqs = new Set<number>()
  const outOfOrderBuffer: WorkflowEvent[] = []
  let eventSource: EventSource | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let reconnectBackoffMs = 1000
  let isTerminalReached = false
  let throttleRunSnapshotTimer: ReturnType<typeof setTimeout> | null = null
  let throttleTaskGraphTimer: ReturnType<typeof setTimeout> | null = null
  let throttleDelegationGraphTimer: ReturnType<typeof setTimeout> | null = null
  let throttleAgentsTimer: ReturnType<typeof setTimeout> | null = null
  let throttleArtifactsTimer: ReturnType<typeof setTimeout> | null = null
  let throttleLogsTimer: ReturnType<typeof setTimeout> | null = null
  let lastLogsFetchTs = 0
  let logsNextCursor: string | null = null
  let inFlightRecovery: Promise<void> | null = null
  let instance!: WorkflowRunInstance
  const scheduleRunSnapshotFetch = () => {
    if (throttleRunSnapshotTimer) return
    throttleRunSnapshotTimer = setTimeout(async () => {
      throttleRunSnapshotTimer = null
      try {
        const freshRun = await service.getWorkflowRun(runId)
        run.value = freshRun
        sectionErrors.value.run = null
        const totalTasks = freshRun.progress?.total_tasks ?? 0
        if (totalTasks > taskGraph.value.nodes.length) {
          scheduleTaskGraphFetch()
        }
      } catch (err: unknown) {
        sectionErrors.value.run = err instanceof Error ? err.message : String(err)
      }
    }, 500)
  }

  const scheduleTaskGraphFetch = () => {
    if (throttleTaskGraphTimer) return
    throttleTaskGraphTimer = setTimeout(async () => {
      throttleTaskGraphTimer = null
      try {
        const freshGraph = await service.getWorkflowTaskGraph(runId)
        taskGraph.value = freshGraph
        sectionErrors.value.taskGraph = null
      } catch (err: unknown) {
        sectionErrors.value.taskGraph = err instanceof Error ? err.message : String(err)
      }
    }, 500)
  }

  const scheduleDelegationGraphFetch = () => {
    if (throttleDelegationGraphTimer) return
    throttleDelegationGraphTimer = setTimeout(async () => {
      throttleDelegationGraphTimer = null
      try {
        const freshGraph = await service.getWorkflowDelegationGraph(runId)
        delegationGraph.value = freshGraph
        sectionErrors.value.delegationGraph = null
      } catch (err: unknown) {
        sectionErrors.value.delegationGraph = err instanceof Error ? err.message : String(err)
      }
    }, 500)
  }

  const scheduleAgentsFetch = () => {
    if (throttleAgentsTimer) return
    throttleAgentsTimer = setTimeout(async () => {
      throttleAgentsTimer = null
      try {
        const freshAgents = await service.getWorkflowAgents(runId)
        agents.value = freshAgents
        sectionErrors.value.agents = null
      } catch (err: unknown) {
        sectionErrors.value.agents = err instanceof Error ? err.message : String(err)
      }
    }, 500)
  }

  const scheduleArtifactsFetch = () => {
    if (throttleArtifactsTimer) return
    throttleArtifactsTimer = setTimeout(async () => {
      throttleArtifactsTimer = null
      try {
        const freshArtifacts = await service.getWorkflowArtifacts(runId)
        artifacts.value = freshArtifacts
        sectionErrors.value.artifacts = null
      } catch (err: unknown) {
        sectionErrors.value.artifacts = err instanceof Error ? err.message : String(err)
      }
    }, 500)
  }
  const mergeLogs = (newLogs: LogEntry[], _prepend = false) => {
    const existingMap = new Map<string, LogEntry>()
    for (const l of logs.value) {
      const key = l.id !== undefined && l.id !== null ? String(l.id) : (l.seq !== undefined && l.seq !== null ? `seq_${l.seq}` : `${l.ts}_${l.message}`)
      existingMap.set(key, l)
    }
    for (const l of newLogs) {
      const key = l.id !== undefined && l.id !== null ? String(l.id) : (l.seq !== undefined && l.seq !== null ? `seq_${l.seq}` : `${l.ts}_${l.message}`)
      existingMap.set(key, l)
    }
    const merged = Array.from(existingMap.values()).sort((a, b) => {
      const tsA = a.ts || ''
      const tsB = b.ts || ''
      if (tsA !== tsB) {
        return tsA.localeCompare(tsB)
      }
      const idA = a.id !== undefined && a.id !== null ? String(a.id) : ''
      const idB = b.id !== undefined && b.id !== null ? String(b.id) : ''
      return idA.localeCompare(idB)
    })
    logs.value = merged
  }

  const scheduleLogsFetch = () => {
    const now = Date.now()
    const timeSinceLast = now - lastLogsFetchTs
    const delay = timeSinceLast >= 2000 ? 0 : 2000 - timeSinceLast

    if (throttleLogsTimer) return
    throttleLogsTimer = setTimeout(async () => {
      throttleLogsTimer = null
      lastLogsFetchTs = Date.now()
      try {
        const res = await service.getWorkflowLogs(runId, { limit: 50 })
        if (res.items) {
          mergeLogs(res.items)
        }
      } catch {
      }
    }, delay)
  }

  const loadMoreLogs = async () => {
    if (!logsNextCursor) return
    try {
      const res = await service.getWorkflowLogs(runId, {
        cursor: String(logsNextCursor),
        limit: 50,
      })
      if (res.items) {
        mergeLogs(res.items)
      }
      logsNextCursor = res.next_cursor !== undefined && res.next_cursor !== null ? String(res.next_cursor) : null
    } catch {
    }
  }

  const fetchFullSnapshot = async () => {
    if (!isInitialized.value) {
      isLoading.value = true
    } else {
      isRefreshing.value = true
    }
    try {
      const [
        runRes,
        taskGraphRes,
        delegationGraphRes,
        agentsRes,
        delegationsRes,
        artifactsRes,
        logsRes,
      ] = await Promise.allSettled([
        service.getWorkflowRun(runId),
        service.getWorkflowTaskGraph(runId),
        service.getWorkflowDelegationGraph(runId),
        service.getWorkflowAgents(runId),
        service.getWorkflowDelegations(runId),
        service.getWorkflowArtifacts(runId),
        service.getWorkflowLogs(runId, { limit: 50 }),
      ])

      if (runRes.status === 'fulfilled') {
        run.value = runRes.value
        sectionErrors.value.run = null
        if (typeof runRes.value.last_seq === 'number' && runRes.value.last_seq > lastSeq.value) {
          lastSeq.value = runRes.value.last_seq
        }
      } else {
        sectionErrors.value.run = runRes.reason instanceof Error ? runRes.reason.message : String(runRes.reason)
      }

      if (taskGraphRes.status === 'fulfilled') {
        taskGraph.value = taskGraphRes.value
        sectionErrors.value.taskGraph = null
      } else {
        sectionErrors.value.taskGraph = taskGraphRes.reason instanceof Error ? taskGraphRes.reason.message : String(taskGraphRes.reason)
      }

      if (delegationGraphRes.status === 'fulfilled') {
        delegationGraph.value = delegationGraphRes.value
        sectionErrors.value.delegationGraph = null
      } else {
        sectionErrors.value.delegationGraph = delegationGraphRes.reason instanceof Error ? delegationGraphRes.reason.message : String(delegationGraphRes.reason)
      }

      if (agentsRes.status === 'fulfilled') {
        agents.value = agentsRes.value
        sectionErrors.value.agents = null
      } else {
        sectionErrors.value.agents = agentsRes.reason instanceof Error ? agentsRes.reason.message : String(agentsRes.reason)
      }

      if (delegationsRes.status === 'fulfilled') {
        delegations.value = delegationsRes.value
        sectionErrors.value.delegations = null
      } else {
        sectionErrors.value.delegations = delegationsRes.reason instanceof Error ? delegationsRes.reason.message : String(delegationsRes.reason)
      }

      if (artifactsRes.status === 'fulfilled') {
        artifacts.value = artifactsRes.value
        sectionErrors.value.artifacts = null
      } else {
        sectionErrors.value.artifacts = artifactsRes.reason instanceof Error ? artifactsRes.reason.message : String(artifactsRes.reason)
      }

      if (logsRes.status === 'fulfilled') {
        if (logsRes.value.items) {
          mergeLogs(logsRes.value.items)
        }
        logsNextCursor = logsRes.value.next_cursor !== undefined && logsRes.value.next_cursor !== null ? String(logsRes.value.next_cursor) : null
      }

      try {
        const targetLastSeq = typeof run.value?.last_seq === 'number' ? run.value.last_seq : lastSeq.value
        const afterSeq = Math.max(0, targetLastSeq - 100)
        const historyRes = await service.getEventsHistory(runId, {
          after_seq: afterSeq,
          limit: 100,
        })
        if (historyRes.events && historyRes.events.length > 0) {
          for (const ev of historyRes.events) {
            if (!seenSeqs.has(ev.seq)) {
              seenSeqs.add(ev.seq)
              events.value.push(ev)
            }
          }
          events.value.sort((a, b) => a.seq - b.seq)
        }
        sectionErrors.value.timeline = null
      } catch (err: unknown) {
        sectionErrors.value.timeline = err instanceof Error ? err.message : String(err)
      }
    } finally {
      isLoading.value = false
      isRefreshing.value = false
      isInitialized.value = true
    }
  }

  const loadOlderEvents = async () => {
    const oldestLoaded = events.value.length > 0 ? events.value[0].seq : lastSeq.value
    if (oldestLoaded <= 1) return
    try {
      const afterSeq = Math.max(0, oldestLoaded - 101)
      const limit = Math.min(100, oldestLoaded - 1)
      const res = await service.getEventsHistory(runId, {
        after_seq: afterSeq,
        limit,
      })
      if (res.events && res.events.length > 0) {
        for (const ev of res.events) {
          if (!seenSeqs.has(ev.seq)) {
            seenSeqs.add(ev.seq)
            events.value.push(ev)
          }
        }
        events.value.sort((a, b) => a.seq - b.seq)
      }
      sectionErrors.value.timeline = null
    } catch (err: unknown) {
      sectionErrors.value.timeline = err instanceof Error ? err.message : String(err)
    }
  }

  const loadEventsHistory = async (reset = false) => {
    try {
      let afterSeq = reset ? 0 : lastSeq.value
      if (reset) {
        events.value = []
        seenSeqs.clear()
      }
      const res = await service.getEventsHistory(runId, {
        after_seq: afterSeq,
        limit: 100,
      })
      if (res.events && res.events.length > 0) {
        for (const ev of res.events) {
          if (!seenSeqs.has(ev.seq)) {
            seenSeqs.add(ev.seq)
            if (ev.seq > lastSeq.value) {
              lastSeq.value = ev.seq
            }
            applyOrderedEvent(ev)
          }
        }
      }
      sectionErrors.value.history = null
    } catch (err: unknown) {
      sectionErrors.value.history = err instanceof Error ? err.message : String(err)
    }
  }

  const patchTaskStatus = (taskId: string, status: TaskStatus, error?: string | null) => {
    const existingNode = taskGraph.value.nodes.find((n) => n.id === taskId)
    if (existingNode) {
      existingNode.data = {
        ...existingNode.data,
        status,
        ...(error !== undefined ? { error } : {}),
      }
    }
  }

  const applyOrderedEvent = (ev: WorkflowEvent) => {
    events.value.push(ev)
    if (ev.type && (
      ev.type.startsWith('task.') ||
      ev.type.startsWith('agent.') ||
      ev.type.startsWith('tool.') ||
      ev.type.startsWith('workflow.')
    )) {
      scheduleLogsFetch()
    }

    switch (ev.type) {
      case 'workflow.started':
        if (run.value) {
          run.value = { ...run.value, status: 'running' }
        }
        break
      case 'workflow.completed':
        if (run.value) {
          const payload = ev.payload as WorkflowCompletedPayload
          run.value = {
            ...run.value,
            status: 'completed',
            completed_at: ev.ts,
            duration_ms: payload.duration_ms ?? run.value.duration_ms,
          }
        }
        break
      case 'workflow.failed':
        if (run.value) {
          const payload = ev.payload as WorkflowFailedPayload
          run.value = {
            ...run.value,
            status: 'failed',
            error_message: payload.error,
          }
        }
        break
      case 'workflow.cancelled':
        if (run.value) {
          const payload = ev.payload as WorkflowCancelledPayload
          run.value = {
            ...run.value,
            status: 'cancelled',
            error_message: payload.reason,
          }
        }
        break
      case 'task.started': {
        const payload = ev.payload as TaskStartedPayload
        const tId = payload.task_id || ev.task_id || ''
        if (tId && !taskGraph.value.nodes.some((n) => n.id === tId)) {
          scheduleTaskGraphFetch()
        }
        patchTaskStatus(tId, 'running')
        scheduleRunSnapshotFetch()
        break
      }
      case 'task.completed': {
        const payload = ev.payload as TaskCompletedPayload
        const tId = payload.task_id || ev.task_id || ''
        if (tId && !taskGraph.value.nodes.some((n) => n.id === tId)) {
          scheduleTaskGraphFetch()
        }
        patchTaskStatus(tId, 'completed')
        scheduleRunSnapshotFetch()
        break
      }
      case 'task.failed': {
        const payload = ev.payload as TaskFailedPayload
        const tId = payload.task_id || ev.task_id || ''
        if (tId && !taskGraph.value.nodes.some((n) => n.id === tId)) {
          scheduleTaskGraphFetch()
        }
        patchTaskStatus(tId, 'failed', payload.error)
        scheduleRunSnapshotFetch()
        break
      }
      case 'task.blocked': {
        const tId = ev.task_id || ''
        if (tId && !taskGraph.value.nodes.some((n) => n.id === tId)) {
          scheduleTaskGraphFetch()
        }
        patchTaskStatus(tId, 'blocked')
        scheduleRunSnapshotFetch()
        break
      }
      case 'task.retrying': {
        const tId = ev.task_id || ''
        if (tId && !taskGraph.value.nodes.some((n) => n.id === tId)) {
          scheduleTaskGraphFetch()
        }
        patchTaskStatus(tId, 'retrying')
        scheduleRunSnapshotFetch()
        break
      }
      case 'task.added': {
        const payload = ev.payload as TaskAddedPayload
        const newTaskId = payload.task_id
        if (newTaskId && !taskGraph.value.nodes.some((n) => n.id === newTaskId)) {
          taskGraph.value = {
            nodes: [
              ...taskGraph.value.nodes,
              {
                id: newTaskId,
                label: payload.name || newTaskId,
                type: 'task',
                data: {
                  status: 'pending',
                  assigned_agent: payload.assigned_agent,
                  attempt: 1,
                  task_type: 'task',
                  required: true,
                  added_in_revision: Number(((payload as unknown) as Record<string, unknown>).added_in_revision),
                },
              },
            ],
            edges: [
              ...taskGraph.value.edges,
              ...(payload.dependencies || []).map((depId: string) => ({
                id: `edge_${depId}_${newTaskId}`,
                source: depId,
                target: newTaskId,
                kind: 'dependency' as const,
              })),
            ],
          }
        }
        scheduleTaskGraphFetch()
        scheduleRunSnapshotFetch()
        break
      }
      case 'plan.revised':
        scheduleTaskGraphFetch()
        scheduleRunSnapshotFetch()
        break
      case 'agent.started': {
        const payload = ev.payload as AgentStartedPayload
        const execId = payload.execution_id || ev.execution_id
        if (execId && !agents.value.some((a) => a.id === execId)) {
          scheduleAgentsFetch()
        }
        break
      }
      case 'agent.completed': {
        const payload = ev.payload as AgentCompletedPayload
        const execId = payload.execution_id || ev.execution_id
        const agent = agents.value.find((a) => a.id === execId)
        if (agent) {
          agent.status = 'completed'
          agent.completed_at = ev.ts
          agent.public_summary = payload.summary || ''
        } else {
          scheduleAgentsFetch()
        }
        break
      }
      case 'agent.failed': {
        const payload = ev.payload as AgentFailedPayload
        const execId = payload.execution_id || ev.execution_id
        const agent = agents.value.find((a) => a.id === execId)
        if (agent) {
          agent.status = 'failed'
          agent.completed_at = ev.ts
          agent.error = payload.error || ''
        } else {
          scheduleAgentsFetch()
        }
        break
      }
      case 'artifact.created': {
        scheduleArtifactsFetch()
        break
      }
      case 'delegation.started':
      case 'delegation.accepted':
      case 'delegation.completed':
      case 'delegation.failed': {
        const payload = ((ev.payload || {}) as unknown) as Record<string, unknown>
        const childExecId = typeof payload.child_execution_id === 'string' ? payload.child_execution_id : ''
        const delId = typeof payload.delegation_id === 'string' ? payload.delegation_id : ''
        const hasNode = childExecId ? delegationGraph.value.nodes.some((n) => n.id === childExecId) : false
        const hasEdge = delId && delegationGraph.value.edges.some((e) => e.id === delId || e.data?.delegation_id === delId)
        if (!hasNode || !hasEdge) {
          scheduleDelegationGraphFetch()
        }
        break
      }
    }
  }

  const drainBuffer = () => {
    outOfOrderBuffer.sort((a, b) => a.seq - b.seq)
    while (outOfOrderBuffer.length > 0) {
      const next = outOfOrderBuffer[0]
      if (next.seq === lastSeq.value + 1) {
        outOfOrderBuffer.shift()
        lastSeq.value = next.seq
        applyOrderedEvent(next)
      } else if (next.seq <= lastSeq.value) {
        outOfOrderBuffer.shift()
      } else {
        break
      }
    }
  }

  const recoverGapOrJump = async (_expectedSeq: number, incomingSeq: number): Promise<void> => {
    if (inFlightRecovery) {
      return inFlightRecovery
    }

    inFlightRecovery = (async () => {
      try {
        let currentAfter = lastSeq.value
        let keepFetching = true

        while (keepFetching && currentAfter < incomingSeq) {
          const historyRes = await service.getEventsHistory(runId, {
            after_seq: currentAfter,
            limit: 100,
          })
          if (!historyRes.events || historyRes.events.length === 0) {
            keepFetching = false
            break
          }

          for (const ev of historyRes.events) {
            if (!seenSeqs.has(ev.seq) && ev.seq > lastSeq.value) {
              seenSeqs.add(ev.seq)
              outOfOrderBuffer.push(ev)
            }
          }

          currentAfter = historyRes.next_after_seq ?? historyRes.last_seq ?? incomingSeq
          if (!historyRes.next_after_seq || currentAfter >= incomingSeq) {
            keepFetching = false
          }
        }

        drainBuffer()
        if (outOfOrderBuffer.length > 0 && outOfOrderBuffer[0].seq > lastSeq.value + 1) {
          await instance.fetchFullSnapshot()
          outOfOrderBuffer.length = 0
          if (run.value?.last_seq !== undefined) {
            lastSeq.value = run.value.last_seq
          }
          await instance.loadEventsHistory(true)
        }
      } catch (err: unknown) {
        sectionErrors.value.history = err instanceof Error ? err.message : String(err)
        await instance.fetchFullSnapshot()
        outOfOrderBuffer.length = 0
        if (run.value?.last_seq !== undefined) {
          lastSeq.value = run.value.last_seq
        }
        await instance.loadEventsHistory(true)
      } finally {
        inFlightRecovery = null
      }
    })()

    return inFlightRecovery
  }

  const processIncomingEvent = (ev: WorkflowEvent): Promise<void> | void => {
    if (seenSeqs.has(ev.seq) || ev.seq <= lastSeq.value) {
      return
    }
    seenSeqs.add(ev.seq)

    const expectedSeq = lastSeq.value + 1
    let recoveryPromise: Promise<void> | void = undefined

    if (ev.seq === expectedSeq) {
      lastSeq.value = ev.seq
      applyOrderedEvent(ev)
      drainBuffer()
    } else if (ev.seq > expectedSeq) {
      outOfOrderBuffer.push(ev)
      recoveryPromise = recoverGapOrJump(expectedSeq, ev.seq)
    }

    if (ev.type && isTerminalEventType(ev.type)) {
      isTerminalReached = true
      disconnectSse()
      scheduleRunSnapshotFetch()
      scheduleArtifactsFetch()
    }
    return recoveryPromise
  }

  const disconnectSse = () => {
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    isConnected.value = false
  }

  const connectSse = (eventSourceFactory?: (url: string) => EventSource) => {
    if (isTerminalReached || typeof window === 'undefined') return
    disconnectSse()

    const sseUrl = `/api/backend/workflows/runs/${runId}/events?after_seq=${lastSeq.value}`
    eventSource = eventSourceFactory ? eventSourceFactory(sseUrl) : new EventSource(sseUrl)

    eventSource.onopen = () => {
      isConnected.value = true
      connectionError.value = null
      reconnectBackoffMs = 1000
    }

    const handleRawEventData = (rawData: unknown, fallbackType?: string) => {
      try {
        let payloadToParse: unknown = rawData
        if (typeof rawData === 'string') {
          payloadToParse = JSON.parse(rawData)
        }
        if (payloadToParse && typeof payloadToParse === 'object' && !('type' in payloadToParse) && fallbackType) {
          (payloadToParse as Record<string, unknown>).type = fallbackType
        }
        const parsed = parseWorkflowEvent(payloadToParse)
        if (parsed) {
          processIncomingEvent(parsed)
        }
      } catch (err) {
        console.warn('[WorkflowStore] Failed to parse SSE event data:', err)
      }
    }

    for (const eventTypeName of WORKFLOW_EVENT_TYPES) {
      eventSource.addEventListener(eventTypeName, (event: MessageEvent) => {
        handleRawEventData(event.data, eventTypeName)
      })
    }

    eventSource.onmessage = (messageEvent) => {
      handleRawEventData(messageEvent.data)
    }

    eventSource.onerror = async () => {
      isConnected.value = false
      connectionError.value = 'SSE connection lost. Reconnecting...'
      disconnectSse()

      if (isTerminalReached) return

      try {
        await service.getWorkflowRun(runId)
      } catch (err: unknown) {
        if (err && typeof err === 'object') {
          const status = 'statusCode' in err ? err.statusCode : ('status' in err ? err.status : undefined)
          if (status === 401) {
            connectionError.value = 'Session expired. Please log in.'
            if (typeof window !== 'undefined') {
              window.location.href = '/login'
            }
            return
          }
          if (status === 404) {
            connectionError.value = 'Run not found or not accessible.'
            sectionErrors.value.run = 'Run not found or not accessible.'
            isTerminalReached = true
            return
          }
        }
      }

      reconnectTimer = setTimeout(() => {
        reconnectBackoffMs = Math.min(reconnectBackoffMs * 1.5, 10000)
        connectSse(eventSourceFactory)
      }, reconnectBackoffMs)
    }
  }
  instance = {
    run,
    taskGraph,
    delegationGraph,
    agents,
    delegations,
    artifacts,
    logs,
    events,
    lastSeq,
    isConnected,
    connectionError,
    isLoading,
    isInitialized,
    isRefreshing,
    sectionErrors,
    selectedTaskId,
    selectedAgentId,
    activeTab,
    graphMode,
    service,
    fetchFullSnapshot,
    loadEventsHistory,
    loadOlderEvents,
    loadMoreLogs,
    connectSse,
    disconnectSse,
    processIncomingEvent,
    patchTaskStatus,
  }

  return instance
}

const activeStores = new Map<string, { store: WorkflowRunInstance; subscribers: number }>()

export function useWorkflowRunStore(runId: string | Ref<string>): {
  store: ShallowRef<WorkflowRunInstance | null>
  runId: Ref<string>
} {
  const resolvedRunId = computed(() => (typeof runId === 'string' ? runId : runId.value))
  const storeRef = shallowRef<WorkflowRunInstance | null>(null)

  const getOrCreateStore = (id: string): WorkflowRunInstance => {
    let entry = activeStores.get(id)
    if (!entry) {
      entry = { store: createRunStoreInstance(id), subscribers: 0 }
      activeStores.set(id, entry)
    }
    return entry.store
  }

  const isClientOrTest = import.meta.client || typeof window !== 'undefined' || process.env.NODE_ENV === 'test'
  if (isClientOrTest) {
    if (resolvedRunId.value) {
      storeRef.value = getOrCreateStore(resolvedRunId.value)
    }
    if (getCurrentInstance()) {
      onMounted(() => {
        const id = resolvedRunId.value
        if (id) {
          const entry = activeStores.get(id)
          if (entry) {
            entry.subscribers++
          }
        }
      })

      onUnmounted(() => {
        const id = resolvedRunId.value
        if (id) {
          const entry = activeStores.get(id)
          if (entry) {
            entry.subscribers--
            if (entry.subscribers <= 0) {
              entry.store.disconnectSse()
              activeStores.delete(id)
            }
          }
        }
      })
    }
  }

  return {
    store: storeRef,
    runId: resolvedRunId,
  }
}
