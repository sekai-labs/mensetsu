import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import Ajv from 'ajv'
import fs from 'node:fs'
import path from 'node:path'
import TaskNode from '../components/workflow/TaskNode.vue'
import DelegationNode from '../components/workflow/DelegationNode.vue'
import ActivityTimeline from '../components/workflow/ActivityTimeline.vue'
import ArtifactsViewer from '../components/workflow/ArtifactsViewer.vue'
import { useWorkflowRunStore } from '../composables/useWorkflowStore'
import { parseNdjsonEvents, parseSseStream } from '../features/workflows/api/contract'
import type {
  WorkflowEvent,
  Artifact,
} from '../models/generated/workflows.v1'

const schemaPath = path.resolve(__dirname, '../../../contracts/rest/schema/workflows.v1.json')
const rawSchema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'))
const ajv = new Ajv({ strict: false, allErrors: true })
ajv.addSchema(rawSchema, 'workflows.v1.json')

const fixturesDir = path.resolve(__dirname, '../../../contracts/rest/fixtures/workflows.v1')

function validateSample(defName: string, data: unknown) {
  const validate = ajv.getSchema(`workflows.v1.json#/$defs/${defName}`)
  if (!validate) {
    throw new Error(`Definition ${defName} not found`)
  }
  const valid = validate(data)
  expect(valid).toBe(true)
}

class FakeEventSource {
  public url: string
  public onopen: (() => void) | null = null
  public onmessage: ((e: MessageEvent) => void) | null = null
  public onerror: (() => void) | null = null
  private listeners: Map<string, Array<(e: MessageEvent) => void>> = new Map()

  constructor(url: string) {
    this.url = url
    queueMicrotask(() => {
      if (this.onopen) this.onopen()
    })
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

  dispatchNamedEvent(type: string, data: string) {
    const event = new MessageEvent(type, { data })
    const list = this.listeners.get(type)
    if (list) {
      for (const listener of list) {
        listener(event)
      }
    }
  }

  close() {}
}

describe('Workflow Execution Studio Components & Store Replaying Fixtures', () => {
  const artifactsFixture = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'artifacts.json'), 'utf8'))
  const eventsNdjson = fs.readFileSync(path.join(fixturesDir, 'events.ndjson'), 'utf8')
  const sseText = fs.readFileSync(path.join(fixturesDir, 'sse.txt'), 'utf8')

  it('renders TaskNode with status pill, duration, and dashed border for optional', () => {
    const nodeData = {
      id: 'task_scrape',
      name: 'Scrape Engineering Blog',
      task_type: 'research',
      assigned_agent: 'company-research',
      status: 'completed' as const,
      duration_ms: 1250,
      attempt: 2,
      required: false,
      added_in_revision: 2,
    }

    const wrapper = mount(TaskNode, {
      props: {
        id: 'task_scrape',
        data: nodeData,
      },
      global: {
        stubs: {
          Handle: true,
        },
      },
    })

    expect(wrapper.text()).toContain('Scrape Engineering Blog')
    expect(wrapper.text()).toContain('company-research')
    expect(wrapper.text()).toContain('completed')
    expect(wrapper.text()).toContain('1.3s')
    expect(wrapper.text()).toContain('att 2')
    expect(wrapper.text()).toContain('rev 2')
    expect(wrapper.text()).toContain('opt')
    expect(wrapper.classes()).toContain('border-dashed')
  })

  it('renders DelegationNode strictly from delegation contract data', () => {
    const nodeData = {
      status: 'running' as const,
      task_id: 'task_verify_claim',
      agent_id: 'evidence-evaluator',
    }

    validateSample('DelegationGraphNodeData', nodeData)

    const wrapper = mount(DelegationNode, {
      props: {
        id: 'exec_evaluator_1',
        label: 'evidence-evaluator',
        data: nodeData,
      },
      global: {
        stubs: {
          Handle: true,
        },
      },
    })

    expect(wrapper.text()).toContain('evidence-evaluator')
    expect(wrapper.text()).toContain('running')
    expect(wrapper.text()).toContain('task_verify_claim')
  })

  it('replays real events.ndjson fixture through adapter and activity timeline', async () => {
    const events: WorkflowEvent[] = parseNdjsonEvents(eventsNdjson)
    expect(events.length).toBeGreaterThan(0)

    const wrapper = mount(ActivityTimeline, {
      props: {
        events,
      },
    })

    expect(wrapper.text()).toContain('workflow.created')
    expect(wrapper.text()).toContain('task.started')
    expect(wrapper.text()).toContain('task.completed')

    const searchInput = wrapper.find('input[type="text"]')
    await searchInput.setValue('company-research')

    const listText = wrapper.find('.divide-y').text()
    expect(listText).toContain('company-research')
  })

  it('displays ArtifactsViewer with real artifacts fixture and provenance evidence', () => {
    const artifacts: Artifact[] = artifactsFixture
    expect(artifacts.length).toBeGreaterThan(0)

    for (const art of artifacts) {
      validateSample('Artifact', art)
    }

    const wrapper = mount(ArtifactsViewer, {
      props: {
        artifacts,
      },
    })

    expect(wrapper.text()).toContain(artifacts[0].title)
  })

  it('replays real sse.txt fixture stream through adapter and store', async () => {
    const { store } = useWorkflowRunStore('test_run_sse_fixture')
    const instance = store.value!

    let fakeSourceInstance: FakeEventSource | null = null
    instance.connectSse((url) => {
      fakeSourceInstance = new FakeEventSource(url)
      return fakeSourceInstance as unknown as EventSource
    })

    const frames = parseSseStream(sseText)
    expect(frames.length).toBeGreaterThan(0)

    for (const frame of frames) {
      if (frame.event && frame.data) {
        fakeSourceInstance!.dispatchNamedEvent(frame.event, JSON.stringify(frame.data))
      }
    }

    expect(instance.events.value.length).toBeGreaterThan(0)
    expect(instance.lastSeq.value).toBe(frames[frames.length - 1].id)
  })

  it('drops stale events with seq <= lastSeq and handles out of order buffering', async () => {
    const { store } = useWorkflowRunStore('test_run_store_1')
    const instance = store.value!
    instance.lastSeq.value = 5

    const staleEvent: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_stale',
      run_id: 'test_run_store_1',
      seq: 4,
      ts: '2026-10-12T10:00:00Z',
      type: 'task.started',
      task_id: 'task_a',
      execution_id: 'exec_a',
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        task_id: 'task_a',
        assigned_agent: 'agent_a',
        attempt: 1,
      },
    }

    instance.processIncomingEvent(staleEvent)
    expect(instance.events.value).toHaveLength(0)
    expect(instance.lastSeq.value).toBe(5)
  })

  it('performs snapshot resync when history gap has an unfillable hole', async () => {
    const { store } = useWorkflowRunStore('test_run_hole')
    const instance = store.value!
    instance.lastSeq.value = 2

    const fetchSnapshotSpy = vi.spyOn(instance, 'fetchFullSnapshot')

    const jumpEvent: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_jump',
      run_id: 'test_run_hole',
      seq: 10,
      ts: '2026-10-12T10:00:00Z',
      type: 'task.started',
      task_id: 'task_x',
      execution_id: 'exec_x',
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        task_id: 'task_x',
        assigned_agent: 'agent_x',
        attempt: 1,
      },
    }
    await instance.processIncomingEvent(jumpEvent)
    expect(fetchSnapshotSpy).toHaveBeenCalled()
  })

  it('guards concurrent gap recovery with single in-flight lock', async () => {
    const { store } = useWorkflowRunStore('test_run_concur')
    const instance = store.value!
    instance.lastSeq.value = 1

    const event5: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_5',
      run_id: 'test_run_concur',
      seq: 5,
      ts: '2026-10-12T10:00:00Z',
      type: 'task.started',
      task_id: 'task_1',
      execution_id: 'exec_1',
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        task_id: 'task_1',
        assigned_agent: 'agent_1',
        attempt: 1,
      },
    }

    const event6: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_6',
      run_id: 'test_run_concur',
      seq: 6,
      ts: '2026-10-12T10:00:05Z',
      type: 'task.completed',
      task_id: 'task_1',
      execution_id: 'exec_1',
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        task_id: 'task_1',
        duration_ms: 100,
        output_summary: 'Done',
      },
    }

    instance.processIncomingEvent(event5)
    instance.processIncomingEvent(event6)
    expect(instance.events.value).toHaveLength(0)
  })

  it('stops and disconnects on terminal event', () => {
    const { store } = useWorkflowRunStore('test_run_terminal')
    const instance = store.value!

    instance.lastSeq.value = 5
    const terminalEv: WorkflowEvent = {
      schema_version: '1.0',
      event_id: 'evt_term',
      run_id: 'test_run_terminal',
      seq: 6,
      ts: '2026-10-12T10:00:30Z',
      type: 'workflow.completed',
      task_id: null,
      execution_id: null,
      parent_execution_id: null,
      tool_invocation_id: null,
      correlation_id: null,
      trace_id: null,
      payload: {
        duration_ms: 30000,
        total_tasks: 5,
        artifacts_count: 2,
      },
    }

    instance.processIncomingEvent(terminalEv)
    expect(instance.lastSeq.value).toBe(6)
    expect(instance.isConnected.value).toBe(false)
  })
})
