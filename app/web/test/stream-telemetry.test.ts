import { describe, it, expect, vi } from 'vitest'
import { useWorkflowRunStore } from '../composables/useWorkflowStore'
import * as workflowServiceModule from '../services/workflow.service'
import type { WorkflowEvent, WorkflowRun, TaskGraphResponse } from '../models/generated/workflows.v1'

class FakeEventSourceForStream {
  public url: string
  public onopen: (() => void) | null = null
  public onmessage: ((e: MessageEvent) => void) | null = null
  public onerror: (() => void) | null = null
  private listeners: Map<string, Array<(e: MessageEvent) => void>> = new Map()

  constructor(url: string) {
    this.url = url
  }

  openSync() {
    if (this.onopen) this.onopen()
  }

  addEventListener(type: string, listener: (e: MessageEvent) => void) {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, [])
    }
    this.listeners.get(type)!.push(listener)
  }

  removeEventListener(type: string, listener: (e: MessageEvent) => void) {
    const list = this.listeners.get(type)
    if (list) {
      this.listeners.set(type, list.filter((l) => l !== listener))
    }
  }

  close() {}

  dispatchNamedEvent(type: string, data: string) {
    const event = new MessageEvent(type, { data })
    const list = this.listeners.get(type)
    if (list) {
      for (const listener of list) {
        listener(event)
      }
    }
    if (type === 'message' && this.onmessage) {
      this.onmessage(event)
    }
  }
}

describe('useWorkflowRunStore Telemetry and Stream Seeding', () => {
  it('requests newest page (after_seq=300), preserves authoritative snapshot state, supports loadOlderEvents (201-300), and applies live 401', async () => {
    const runSnapshot: WorkflowRun = {
      id: 'run_test_400',
      user_id: 'usr_test',
      workflow_name: 'Test Workflow',
      workflow_id: 'interview-preparation-static',
      workspace_id: 'ws_test',
      status: 'running',
      type: 'static',
      current_phase: 'researching_company',
      last_seq: 400,
      created_at: '2026-10-14T00:00:00Z',
    }

    const taskGraphSnapshot: TaskGraphResponse = {
      nodes: [
        {
          id: 'task_research',
          label: 'Company Research',
          type: 'task',
          data: {
            status: 'completed',
            assigned_agent: 'company-research',
            attempt: 1,
            task_type: 'task',
            required: true,
            added_in_revision: 1,
          },
        },
      ],
      edges: [],
    }

    const historyEventsPage301to400: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_350',
        run_id: 'run_test_400',
        seq: 350,
        ts: '2026-10-14T00:00:10Z',
        type: 'task.started',
        task_id: 'task_research',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          task_id: 'task_research',
          attempt: 1,
          assigned_agent: 'company-research',
        },
      },
      {
        schema_version: '1.0',
        event_id: 'evt_400',
        run_id: 'run_test_400',
        seq: 400,
        ts: '2026-10-14T00:00:20Z',
        type: 'agent.activity',
        task_id: 'task_research',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          execution_id: 'exec_1',
          activity_type: 'model_step',
          description: 'Evaluating candidate evidence',
        },
      },
    ]

    const olderEventsPage201to300: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_250',
        run_id: 'run_test_400',
        seq: 250,
        ts: '2026-10-14T00:00:05Z',
        type: 'agent.started',
        task_id: 'task_research',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          execution_id: 'exec_1',
          agent_id: 'company-research',
          model: 'claude-3-5-sonnet',
          objective: 'Research target organization',
        },
      },
    ]

    const getWorkflowRunSpy = vi.fn().mockResolvedValue(runSnapshot)
    const getEventsHistorySpy = vi.fn().mockImplementation((_runId, params) => {
      if (params?.after_seq === 300) {
        return Promise.resolve({
          events: historyEventsPage301to400,
          last_seq: 400,
        })
      }
      if (params?.after_seq <= 250) {
        return Promise.resolve({
          events: olderEventsPage201to300,
          last_seq: 300,
        })
      }
      return Promise.resolve({ events: [], last_seq: 400 })
    })

    const getTaskGraphSpy = vi.fn().mockResolvedValue(taskGraphSnapshot)
    const getDelegationGraphSpy = vi.fn().mockResolvedValue({ nodes: [], edges: [] })
    const getAgentsSpy = vi.fn().mockResolvedValue([])
    const getDelegationsSpy = vi.fn().mockResolvedValue([])
    const getArtifactsSpy = vi.fn().mockResolvedValue([])
    const getLogsSpy = vi.fn().mockResolvedValue({ items: [] })

    vi.spyOn(workflowServiceModule, 'useWorkflowService').mockReturnValue({
      getWorkflowRun: getWorkflowRunSpy,
      getEventsHistory: getEventsHistorySpy,
      getWorkflowTaskGraph: getTaskGraphSpy,
      getWorkflowDelegationGraph: getDelegationGraphSpy,
      getWorkflowAgents: getAgentsSpy,
      getWorkflowDelegations: getDelegationsSpy,
      getWorkflowArtifacts: getArtifactsSpy,
      getWorkflowLogs: getLogsSpy,
      listWorkflowRuns: vi.fn(),
      createWorkflowRun: vi.fn(),
      getAgentTools: vi.fn(),
      getToolOutput: vi.fn(),
      getAgentExecution: vi.fn(),
      cancelWorkflowRun: vi.fn(),
    })

    const { store } = useWorkflowRunStore('run_test_400')
    const instance = store.value!

    await instance.fetchFullSnapshot()

    expect(getEventsHistorySpy).toHaveBeenCalledWith('run_test_400', {
      after_seq: 300,
      limit: 100,
    })

    expect(instance.lastSeq.value).toBe(400)
    expect(instance.events.value).toHaveLength(2)

    const taskNode = instance.taskGraph.value.nodes.find((n) => n.id === 'task_research')
    expect(taskNode?.data.status).toBe('completed')

    await instance.loadOlderEvents()
    expect(getEventsHistorySpy).toHaveBeenCalledWith('run_test_400', {
      after_seq: 249,
      limit: 100,
    })
    expect(instance.events.value).toHaveLength(3)
    expect(instance.events.value[0].seq).toBe(250)

    let capturedUrl = ''
    let fakeSource: FakeEventSourceForStream | null = null

    instance.connectSse((url) => {
      capturedUrl = url
      fakeSource = new FakeEventSourceForStream(url)
      return fakeSource as unknown as EventSource
    })

    expect(capturedUrl).toContain('/api/backend/workflows/runs/run_test_400/events?after_seq=400')
    expect(fakeSource).not.toBeNull()
    fakeSource!.openSync()

    const liveEvent401: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_401',
      run_id: 'run_test_400',
      seq: 401,
      ts: '2026-10-14T00:00:30Z',
      type: 'task.completed',
      task_id: 'task_research',
      execution_id: 'exec_1',
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        task_id: 'task_research',
        duration_ms: 10000,
        output_summary: 'Research completed',
      },
    }

    fakeSource!.dispatchNamedEvent('task.completed', JSON.stringify(liveEvent401))

    expect(instance.lastSeq.value).toBe(401)
    expect(instance.events.value).toHaveLength(4)
    expect(instance.events.value[3].seq).toBe(401)
  })
})
