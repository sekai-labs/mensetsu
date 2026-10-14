import type {
  WorkflowRun,
  TaskGraphResponse,
  DelegationGraphResponse,
  RunsPageResponse,
  LogsPageResponse,
  EventsHistoryPageResponse,
  ToolInvocationsPageResponse,
  ToolOutputResponse,
  WorkflowEvent,
} from '~/models/generated/workflows.v1'

export interface ParsedSseFrame {
  id?: number
  event?: string
  data?: unknown
  rawEvent?: WorkflowEvent
}

export const KNOWN_EVENT_TYPES: Record<string, true> = {
  'workflow.created': true,
  'workflow.started': true,
  'workflow.completed': true,
  'workflow.failed': true,
  'workflow.cancelled': true,
  'plan.created': true,
  'plan.revised': true,
  'task.added': true,
  'task.removed': true,
  'task.queued': true,
  'task.started': true,
  'task.completed': true,
  'task.failed': true,
  'task.retrying': true,
  'task.blocked': true,
  'agent.started': true,
  'agent.activity': true,
  'agent.completed': true,
  'agent.failed': true,
  'delegation.requested': true,
  'delegation.accepted': true,
  'delegation.started': true,
  'delegation.completed': true,
  'delegation.failed': true,
  'tool.started': true,
  'tool.completed': true,
  'tool.failed': true,
  'artifact.created': true,
  'finding.saved': true,
  'evidence.saved': true,
  'checkpoint.created': true,
  'execution.resumed': true,
  'worker.recovered': true,
}

export const TERMINAL_EVENT_TYPES: Record<string, true> = {
  'workflow.completed': true,
  'workflow.failed': true,
  'workflow.cancelled': true,
}

export function isTerminalEventType(eventType: string): boolean {
  return Boolean(TERMINAL_EVENT_TYPES[eventType])
}


export function parseWorkflowEvent(raw: unknown): WorkflowEvent | null {
  if (!raw) {
    console.warn('[WorkflowContract] Received empty event payload')
    return null
  }

  let parsed: unknown = raw
  if (typeof raw === 'string') {
    try {
      parsed = JSON.parse(raw)
    } catch (err) {
      console.warn('[WorkflowContract] Failed to parse event JSON:', err, raw)
      return null
    }
  }

  if (typeof parsed !== 'object' || parsed === null) {
    console.warn('[WorkflowContract] Event payload is not an object:', parsed)
    return null
  }

  const envelope = parsed as Record<string, unknown>
  const eventType = envelope.type
  if (typeof eventType !== 'string') {
    console.warn('[WorkflowContract] Event missing "type" field:', envelope)
    return null
  }

  if (!KNOWN_EVENT_TYPES[eventType]) {
    console.warn(`[WorkflowContract] Unknown event type encountered: "${eventType}". Ignoring without crash.`, envelope)
    return null
  }

  if (typeof envelope.event_id !== 'string' || typeof envelope.run_id !== 'string' || typeof envelope.seq !== 'number') {
    console.warn('[WorkflowContract] Event envelope missing required header fields (event_id, run_id, seq):', envelope)
    return null
  }

  return envelope as unknown as WorkflowEvent
}



export function parseSseFrame(frameText: string): ParsedSseFrame | null {
  const lines = frameText.split(/\r?\n/)
  let id: number | undefined
  let event: string | undefined
  const dataLines: string[] = []

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith(':')) {
      continue
    }

    if (trimmed.startsWith('id:')) {
      const parsedId = Number.parseInt(trimmed.slice(3).trim(), 10)
      if (!Number.isNaN(parsedId)) {
        id = parsedId
      }
    } else if (trimmed.startsWith('event:')) {
      event = trimmed.slice(6).trim()
    } else if (trimmed.startsWith('data:')) {
      dataLines.push(trimmed.slice(5).trim())
    }
  }

  if (dataLines.length === 0) {
    return null
  }

  const rawDataStr = dataLines.join('\n')
  let data: unknown
  try {
    data = JSON.parse(rawDataStr)
  } catch (err) {
    console.warn('[WorkflowContract] Failed to parse SSE data JSON:', err, rawDataStr)
    return null
  }

  let rawEvent: WorkflowEvent | undefined
  if (data && typeof data === 'object' && 'type' in data) {
    const parsedEv = parseWorkflowEvent(data)
    if (parsedEv) {
      rawEvent = parsedEv
    }
  }

  return {
    id,
    event,
    data,
    rawEvent,
  }
}



export function parseSseStream(streamText: string): ParsedSseFrame[] {
  const blocks = streamText.split(/(?:\r?\n){2,}/)
  const frames: ParsedSseFrame[] = []

  for (const block of blocks) {
    if (!block.trim()) continue
    const frame = parseSseFrame(block)
    if (frame) {
      frames.push(frame)
    }
  }

  return frames
}



export function parseNdjsonEvents(ndjsonText: string): WorkflowEvent[] {
  const lines = ndjsonText.split(/\r?\n/)
  const events: WorkflowEvent[] = []

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed) continue
    const ev = parseWorkflowEvent(trimmed)
    if (ev) {
      events.push(ev)
    }
  }

  return events
}
export function parseWorkflowRunResponse(raw: unknown): WorkflowRun {
  if (!raw || typeof raw !== 'object' || !('id' in raw)) {
    throw new Error('[WorkflowContract] Invalid WorkflowRun response')
  }
  return raw as WorkflowRun
}

export function parseRunsPageResponse(raw: unknown): RunsPageResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as RunsPageResponse).items)) {
    throw new Error('[WorkflowContract] Invalid RunsPageResponse')
  }
  return raw as RunsPageResponse
}

export function parseTaskGraphResponse(raw: unknown): TaskGraphResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as TaskGraphResponse).nodes)) {
    throw new Error('[WorkflowContract] Invalid TaskGraphResponse')
  }
  return raw as TaskGraphResponse
}

export function parseDelegationGraphResponse(raw: unknown): DelegationGraphResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as DelegationGraphResponse).nodes)) {
    throw new Error('[WorkflowContract] Invalid DelegationGraphResponse')
  }
  return raw as DelegationGraphResponse
}

export function parseEventsHistoryResponse(raw: unknown): EventsHistoryPageResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as EventsHistoryPageResponse).events)) {
    throw new Error('[WorkflowContract] Invalid EventsHistoryPageResponse')
  }
  return raw as EventsHistoryPageResponse
}

export function parseLogsPageResponse(raw: unknown): LogsPageResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as LogsPageResponse).items)) {
    throw new Error('[WorkflowContract] Invalid LogsPageResponse')
  }
  return raw as LogsPageResponse
}

export function parseToolInvocationsPageResponse(raw: unknown): ToolInvocationsPageResponse {
  if (!raw || typeof raw !== 'object' || !Array.isArray((raw as ToolInvocationsPageResponse).items)) {
    throw new Error('[WorkflowContract] Invalid ToolInvocationsPageResponse')
  }
  return raw as ToolInvocationsPageResponse
}

export function parseToolOutputResponse(raw: unknown): ToolOutputResponse {
  if (!raw || typeof raw !== 'object' || typeof (raw as ToolOutputResponse).tool_invocation_id !== 'string') {
    throw new Error('[WorkflowContract] Invalid ToolOutputResponse')
  }
  return raw as ToolOutputResponse
}
