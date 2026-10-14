import { ref, onMounted, onUnmounted, getCurrentInstance } from 'vue'
import type { PipelinePhase, WorkspaceStatus } from '~/models/common.model'
import type { ProgressEventPayload } from '../models/interview.model'

export const useInterviewEventsService = (
  workspaceId: string,
  onPhaseChange?: (payload: ProgressEventPayload) => void,
  eventSourceFactory?: (url: string) => EventSource
) => {
  const currentPhase = ref<PipelinePhase>('preparing_input')
  const progressPercent = ref<number>(0)
  const status = ref<WorkspaceStatus>('pending')
  const isConnected = ref<boolean>(false)
  const message = ref<string>('')

  let eventSource: EventSource | null = null
  let isTerminalReached = false

  const handleProgressData = (dataStr: string) => {
    try {
      const data = JSON.parse(dataStr) as ProgressEventPayload
      if (data.phase) {
        currentPhase.value = data.phase
        if (data.phase === 'ready') {
          status.value = 'ready'
        } else if (data.phase === 'failed') {
          status.value = 'failed'
        } else {
          status.value = 'processing'
        }
      }
      if (typeof data.progress_percent === 'number') {
        progressPercent.value = data.progress_percent
      }
      if (data.message) {
        message.value = data.message
      }

      if (onPhaseChange) {
        onPhaseChange(data)
      }

      if (data.phase === 'ready' || data.phase === 'failed') {
        isTerminalReached = true
        disconnect()
      }
    } catch {
    }
  }

  const connect = () => {
    if (!workspaceId || isTerminalReached || typeof window === 'undefined') return

    const cleanId = workspaceId.replace(/^workspace:/, '').replace(/`/g, '')
    const url = `/api/backend/workspaces/${cleanId}/events`

    disconnect()
    eventSource = eventSourceFactory ? eventSourceFactory(url) : new EventSource(url)

    eventSource.onopen = () => {
      isConnected.value = true
    }

    eventSource.addEventListener('progress', (event: MessageEvent) => {
      handleProgressData(event.data)
    })

    eventSource.onmessage = (event: MessageEvent) => {
      handleProgressData(event.data)
    }

    eventSource.onerror = () => {
      isConnected.value = false
      if (isTerminalReached) {
        disconnect()
      }
    }
  }

  const disconnect = () => {
    if (eventSource) {
      eventSource.close()
      eventSource = null
      isConnected.value = false
    }
  }

  if (getCurrentInstance()) {
    onMounted(() => {
      connect()
    })

    onUnmounted(() => {
      disconnect()
    })
  }
  return {
    currentPhase,
    progressPercent,
    status,
    isConnected,
    message,
    connect,
    disconnect,
  }
}
