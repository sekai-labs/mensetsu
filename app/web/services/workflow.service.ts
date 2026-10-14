import type {
  WorkflowRun,
  AgentExecution,
  Delegation,
  Artifact,
  TaskGraphResponse,
  DelegationGraphResponse,
  RunsPageResponse,
  LogsPageResponse,
  EventsHistoryPageResponse,
  ToolInvocationsPageResponse,
  ToolOutputResponse,
} from '~/models/generated/workflows.v1'
import {
  parseWorkflowRunResponse,
  parseRunsPageResponse,
  parseTaskGraphResponse,
  parseDelegationGraphResponse,
  parseEventsHistoryResponse,
  parseLogsPageResponse,
  parseToolInvocationsPageResponse,
  parseToolOutputResponse,
} from '~/features/workflows/api/contract'
import { useApiClient } from '~/services/api.client'

export interface CreateWorkflowRunPayload {
  workflow_id: string
  workspace_id: string
}

export const useWorkflowService = () => {
  const { request } = useApiClient()

  const listWorkflowRuns = async (params?: {
    workspace_id?: string
    status?: string
    type?: string
    cursor?: string
    limit?: number
  }): Promise<RunsPageResponse> => {
    const raw = await request<unknown>('/api/v1/workflows/runs', {
      method: 'GET',
      query: params,
    })
    return parseRunsPageResponse(raw)
  }

  const createWorkflowRun = async (
    payload: CreateWorkflowRunPayload
  ): Promise<WorkflowRun> => {
    const raw = await request<unknown>('/api/v1/workflows/runs', {
      method: 'POST',
      body: payload,
    })
    return parseWorkflowRunResponse(raw)
  }

  const getWorkflowRun = async (runId: string): Promise<WorkflowRun> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}`, {
      method: 'GET',
    })
    return parseWorkflowRunResponse(raw)
  }

  const getWorkflowTaskGraph = async (runId: string): Promise<TaskGraphResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/graph`, {
      method: 'GET',
    })
    return parseTaskGraphResponse(raw)
  }

  const getWorkflowDelegationGraph = async (runId: string): Promise<DelegationGraphResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/delegations/graph`, {
      method: 'GET',
    })
    return parseDelegationGraphResponse(raw)
  }

  const getWorkflowLogs = async (
    runId: string,
    params?: {
      cursor?: string
      limit?: number
      severity?: string
    }
  ): Promise<LogsPageResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/logs`, {
      method: 'GET',
      query: params,
    })
    return parseLogsPageResponse(raw)
  }

  const getEventsHistory = async (
    runId: string,
    params?: {
      after_seq?: number
      limit?: number
    }
  ): Promise<EventsHistoryPageResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/events/history`, {
      method: 'GET',
      query: params,
    })
    return parseEventsHistoryResponse(raw)
  }

  const getAgentTools = async (
    runId: string,
    executionId: string,
    params?: {
      cursor?: string
      limit?: number
    }
  ): Promise<ToolInvocationsPageResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/agents/${executionId}/tools`, {
      method: 'GET',
      query: params,
    })
    return parseToolInvocationsPageResponse(raw)
  }

  const getToolOutput = async (
    runId: string,
    toolInvocationId: string,
    params?: {
      offset?: number
      limit?: number
    }
  ): Promise<ToolOutputResponse> => {
    const raw = await request<unknown>(`/api/v1/workflows/runs/${runId}/tools/${toolInvocationId}/output`, {
      method: 'GET',
      query: params,
    })
    return parseToolOutputResponse(raw)
  }

  const getWorkflowDelegations = async (runId: string): Promise<Delegation[]> => {
    return await request<Delegation[]>(`/api/v1/workflows/runs/${runId}/delegations`, {
      method: 'GET',
    })
  }

  const getWorkflowAgents = async (runId: string): Promise<AgentExecution[]> => {
    return await request<AgentExecution[]>(`/api/v1/workflows/runs/${runId}/agents`, {
      method: 'GET',
    })
  }

  const getAgentExecution = async (runId: string, executionId: string): Promise<AgentExecution> => {
    return await request<AgentExecution>(`/api/v1/workflows/runs/${runId}/agents/${executionId}`, {
      method: 'GET',
    })
  }

  const getWorkflowArtifacts = async (runId: string): Promise<Artifact[]> => {
    return await request<Artifact[]>(`/api/v1/workflows/runs/${runId}/artifacts`, {
      method: 'GET',
    })
  }

  const cancelWorkflowRun = async (
    runId: string
  ): Promise<WorkflowRun> => {
    const raw = await request<unknown>(
      `/api/v1/workflows/runs/${runId}/cancel`,
      {
        method: 'POST',
      }
    )
    return parseWorkflowRunResponse(raw)
  }

  return {
    listWorkflowRuns,
    createWorkflowRun,
    getWorkflowRun,
    getWorkflowTaskGraph,
    getWorkflowDelegationGraph,
    getWorkflowLogs,
    getEventsHistory,
    getAgentTools,
    getToolOutput,
    getWorkflowDelegations,
    getWorkflowAgents,
    getAgentExecution,
    getWorkflowArtifacts,
    cancelWorkflowRun,
  }
}
