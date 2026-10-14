import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import AgentStepFeed from '../components/workflow/AgentStepFeed.vue'
import type { WorkflowEvent } from '../models/generated/workflows.v1'

describe('AgentStepFeed Component', () => {
  it('pairs tool.started and tool.completed events by tool_invocation_id with duration and summary', () => {
    const events: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_start_1',
        run_id: 'run_1',
        seq: 1,
        ts: '2026-10-13T10:00:01.000Z',
        type: 'tool.started',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: 'tool_inv_123',
        correlation_id: null,
        trace_id: null,
        payload: {
          tool_invocation_id: 'tool_inv_123',
          tool_name: 'web_search',
          sanitized_input: { query: 'Acme engineering' },
        },
      },
      {
        schema_version: '1.0',
        event_id: 'evt_comp_1',
        run_id: 'run_1',
        seq: 2,
        ts: '2026-10-13T10:00:01.450Z',
        type: 'tool.completed',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: 'tool_inv_123',
        correlation_id: null,
        trace_id: null,
        payload: {
          tool_invocation_id: 'tool_inv_123',
          tool_name: 'web_search',
          duration_ms: 450,
          summary: 'Found 3 results for Acme',
        },
      },
    ]

    const wrapper = mount(AgentStepFeed, {
      props: {
        executionId: 'exec_1',
        events,
      },
    })

    const toolRows = wrapper.findAll('[data-testid="step-row-tool"]')
    expect(toolRows).toHaveLength(1)

    const rowText = toolRows[0].text()
    expect(rowText).toContain('web_search')
    expect(rowText).toContain('Acme engineering')
    expect(rowText).toContain('450ms')

    expect(wrapper.find('[data-testid="tool-result-summary"]').exists()).toBe(false)
    const expandBtn = toolRows[0].find('button[title="Expand result"]')
    expect(expandBtn.exists()).toBe(true)
  })

  it('displays an orphan started event as running status', () => {
    const events: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_orphan',
        run_id: 'run_1',
        seq: 10,
        ts: '2026-10-13T10:05:00.000Z',
        type: 'tool.started',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: 'tool_inv_running',
        correlation_id: null,
        trace_id: null,
        payload: {
          tool_invocation_id: 'tool_inv_running',
          tool_name: 'read_source',
          sanitized_input: { url: 'https://example.com/api' },
        },
      },
    ]

    const wrapper = mount(AgentStepFeed, {
      props: {
        executionId: 'exec_1',
        events,
      },
    })

    const toolRows = wrapper.findAll('[data-testid="step-row-tool"]')
    expect(toolRows).toHaveLength(1)
    expect(toolRows[0].text()).toContain('read_source')
    expect(toolRows[0].text()).toContain('https://example.com/api')

    const spinner = toolRows[0].find('.animate-spin')
    expect(spinner.exists()).toBe(true)
  })

  it('orders steps strictly by seq even if events arrive out of order', () => {
    const events: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_third',
        run_id: 'run_1',
        seq: 30,
        ts: '2026-10-13T10:00:30.000Z',
        type: 'agent.activity',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          execution_id: 'exec_1',
          activity_type: 'model_step',
          description: 'Step 3: Finishing up',
        },
      },
      {
        schema_version: '1.0',
        event_id: 'evt_first',
        run_id: 'run_1',
        seq: 10,
        ts: '2026-10-13T10:00:10.000Z',
        type: 'agent.activity',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          execution_id: 'exec_1',
          activity_type: 'model_step',
          description: 'Step 1: Initializing',
        },
      },
      {
        schema_version: '1.0',
        event_id: 'evt_second',
        run_id: 'run_1',
        seq: 20,
        ts: '2026-10-13T10:00:20.000Z',
        type: 'agent.activity',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          execution_id: 'exec_1',
          activity_type: 'model_step',
          description: 'Step 2: Thinking',
        },
      },
    ]

    const wrapper = mount(AgentStepFeed, {
      props: {
        executionId: 'exec_1',
        events,
      },
    })

    const rows = wrapper.findAll('[data-testid="step-row-activity"]')
    expect(rows).toHaveLength(3)
    expect(rows[0].text()).toContain('Step 1: Initializing')
    expect(rows[1].text()).toContain('Step 2: Thinking')
    expect(rows[2].text()).toContain('Step 3: Finishing up')
  })

  it('renders delegation row and links to the child execution id upon click', async () => {
    const events: WorkflowEvent[] = [
      {
        schema_version: '1.0',
        event_id: 'evt_del',
        run_id: 'run_1',
        seq: 15,
        ts: '2026-10-13T10:00:15.000Z',
        type: 'delegation.started',
        task_id: 'task_1',
        execution_id: 'exec_1',
        parent_execution_id: null,
        tool_invocation_id: null,
        correlation_id: null,
        trace_id: null,
        payload: {
          delegation_id: 'del_999',
          child_execution_id: 'exec_child_456',
        },
      },
    ]

    const wrapper = mount(AgentStepFeed, {
      props: {
        executionId: 'exec_1',
        events,
      },
    })

    const delRow = wrapper.find('[data-testid="step-row-delegation"]')
    expect(delRow.exists()).toBe(true)
    expect(delRow.text()).toContain('exec_child_456')

    const button = delRow.find('button')
    expect(button.exists()).toBe(true)
    await button.trigger('click')

    expect(wrapper.emitted('select-execution')).toBeTruthy()
    expect(wrapper.emitted('select-execution')![0]).toEqual(['exec_child_456'])
  })
})
