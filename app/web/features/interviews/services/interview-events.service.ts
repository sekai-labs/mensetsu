import { ref, onMounted, onUnmounted } from 'vue'
import type { ProgressEventPayload } from '../models/interview.model'

/**
 * Service hook to stream real-time workspace pipeline events via Server-Sent Events (SSE).
 */
export const useInterviewEventsService = (
  workspaceId: string,
  onUpdate?: (event: ProgressEventPayload) => void
) => {
  const eventSource = ref<EventSource | null>(null)
  const isConnected = ref(false)
  const currentPhase = ref('preparing_input')
  const progressPercent = ref(0)
  const status = ref('pending')

  onMounted(() => {
    if (!import.meta.client || !workspaceId) return

    const cleanId = workspaceId.replace(/^workspace:/, '').replace(/`/g, '')
    const url = `/api/backend/workspaces/${cleanId}/events`
    const es = new EventSource(url, { withCredentials: true })
    eventSource.value = es

    es.onopen = () => {
      isConnected.value = true
    }

    es.addEventListener('progress', (e: MessageEvent) => {
      try {
        const parsed: unknown = JSON.parse(e.data)
        if (parsed && typeof parsed === 'object' && 'phase' in parsed && 'percent' in parsed) {
          const payload = parsed as ProgressEventPayload
          currentPhase.value = payload.phase
          progressPercent.value = payload.percent
          status.value = payload.status
          onUpdate?.(payload)
        }
      } catch (err) {
        console.error('Failed to parse SSE progress event:', err)
      }
    })

    es.onerror = () => {
      isConnected.value = false
      es.close()
    }
  })

  onUnmounted(() => {
    if (eventSource.value) {
      eventSource.value.close()
      eventSource.value = null
    }
  })

  return {
    isConnected,
    currentPhase,
    progressPercent,
    status,
  }
}
