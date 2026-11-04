import { describe, it, expect, vi } from 'vitest'
import { defineComponent, h, nextTick, ref } from 'vue'
import { mount } from '@vue/test-utils'
import { useInterviewEventsService } from '../features/interviews/services/interview-events.service'

class FakeWorkspaceEventSource {
  public url: string
  public onopen: (() => void) | null = null
  public onmessage: ((e: MessageEvent) => void) | null = null
  public onerror: (() => void) | null = null
  public closed = false
  private listeners: Map<string, Array<(e: MessageEvent) => void>> = new Map()

  constructor(url: string) {
    this.url = url
  }

  openSync() {
    if (!this.closed && this.onopen) {
      this.onopen()
    }
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

  close() {
    this.closed = true
  }

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

describe('useInterviewEventsService Progress Streaming', () => {
  it('updates progress on named progress events, calls refresh on ready, and closes without reconnecting', () => {
    let fakeSource: FakeWorkspaceEventSource | null = null
    const onPhaseChangeSpy = vi.fn()

    const service = useInterviewEventsService(
      'ws_test_123',
      onPhaseChangeSpy,
      (url: string) => {
        fakeSource = new FakeWorkspaceEventSource(url)
        return fakeSource as unknown as EventSource
      }
    )

    service.connect()
    expect(fakeSource).not.toBeNull()

    fakeSource!.openSync()
    expect(service.isConnected.value).toBe(true)

    fakeSource!.dispatchNamedEvent(
      'progress',
      JSON.stringify({
        workspace_id: 'ws_test_123',
        phase: 'analyzing_job',
        progress_percent: 45,
        message: 'Progress update: analyzing_job (45%)',
      })
    )

    expect(service.currentPhase.value).toBe('analyzing_job')
    expect(service.progressPercent.value).toBe(45)
    expect(service.status.value).toBe('processing')
    expect(service.message.value).toBe('Progress update: analyzing_job (45%)')
    expect(onPhaseChangeSpy).toHaveBeenCalledTimes(1)
    expect(fakeSource!.closed).toBe(false)

    fakeSource!.dispatchNamedEvent(
      'progress',
      JSON.stringify({
        workspace_id: 'ws_test_123',
        phase: 'ready',
        progress_percent: 100,
        message: 'Progress update: ready (100%)',
      })
    )

    expect(service.currentPhase.value).toBe('ready')
    expect(service.progressPercent.value).toBe(100)
    expect(service.status.value).toBe('ready')
    expect(onPhaseChangeSpy).toHaveBeenCalledTimes(2)
    expect(fakeSource!.closed).toBe(true)
    expect(service.isConnected.value).toBe(false)

    service.connect()
    expect(fakeSource!.closed).toBe(true)
  })
  it('reconnects when the workspace route changes and clears previous workspace progress', async () => {
    const workspaceId = ref('first')
    const sources: FakeWorkspaceEventSource[] = []
    let progressPercent: { value: number } | undefined
    const component = defineComponent({
      setup() {
        progressPercent = useInterviewEventsService(workspaceId, undefined, (url) => {
          const source = new FakeWorkspaceEventSource(url)
          sources.push(source)
          return source as unknown as EventSource
        }).progressPercent
        return () => h('div')
      },
    })
    const wrapper = mount(component)

    expect(sources.map((source) => source.url)).toEqual(['/api/backend/workspaces/first/events'])
    sources[0]!.dispatchNamedEvent('progress', JSON.stringify({
      workspace_id: 'first',
      phase: 'analyzing_job',
      progress_percent: 45,
    }))

    workspaceId.value = 'second'
    await nextTick()
    expect(progressPercent?.value).toBe(0)

    expect(sources[0]!.closed).toBe(true)
    expect(sources.map((source) => source.url)).toEqual([
      '/api/backend/workspaces/first/events',
      '/api/backend/workspaces/second/events',
    ])
    wrapper.unmount()
    expect(sources[1]!.closed).toBe(true)
  })
})
