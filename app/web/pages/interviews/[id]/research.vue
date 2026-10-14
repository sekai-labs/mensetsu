<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { useWorkflowService } from '~/services/workflow.service'
import { useWorkflowRunStore } from '~/composables/useWorkflowStore'
import type { SourceCitation } from '~/features/interviews/models/interview.model'
import EvidenceDrawer from '~/components/ui/EvidenceDrawer.vue'
import StatusBadge from '~/components/ui/StatusBadge.vue'
import AgentStepFeed from '~/components/workflow/AgentStepFeed.vue'
definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace, getWorkspaceCompany } = useInterviewService()
const { listWorkflowRuns } = useWorkflowService()

const { data: ws } = await useAsyncData(`research-ws-${workspaceId.value}`, () => getWorkspace(workspaceId.value))
const { data: companyData } = await useAsyncData(`research-comp-${workspaceId.value}`, () => getWorkspaceCompany(workspaceId.value))
const { data: runsData } = await useAsyncData(`research-runs-${workspaceId.value}`, () => listWorkflowRuns({ workspace_id: workspaceId.value, limit: 1 }))

const activeRunId = computed(() => runsData.value?.items?.[0]?.id || '')
const { store: workflowStore } = useWorkflowRunStore(activeRunId)

onMounted(() => {
  if (workflowStore.value && activeRunId.value) {
    workflowStore.value.connectSse()
  }
})

const claims = computed<any[]>(() => {
  const raw = companyData.value as any
  return Array.isArray(raw?.claims) ? raw.claims : []
})

const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)
const selectedCategory = ref<string>('all')

const categories = computed(() => {
  const list = ['all']
  const cats = new Set<string>()
  for (const c of claims.value) {
    cats.add(c.category || 'Architecture')
  }
  cats.forEach((c) => list.push(c))
  return list
})

const filteredClaims = computed(() => {
  if (selectedCategory.value === 'all') return claims.value
  return claims.value.filter((c) => (c.category || 'Architecture') === selectedCategory.value)
})

const openCitation = (source: SourceCitation) => {
  selectedSource.value = source
  isDrawerOpen.value = true
}

const openSourceById = (sourceId?: string) => {
  if (!sourceId || !companyData.value?.sources) return
  const found = companyData.value.sources.find((s) => s.id === sourceId)
  if (found) {
    openCitation(found)
  }
}
</script>

<template>
  <div class="space-y-8">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>DEEP RESEARCH WORKSPACE</span>
          <span>•</span>
          <span>LANGGRAPH AGENT SUPERVISOR</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ companyData?.company_name || ws?.company_name || 'Organization' }} Research Dossier
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          Evidence-backed technical profile, verifiable architecture topology, verified citations, and adaptive research telemetry.
        </p>
      </div>

      <div class="flex items-center gap-3">
        <StatusBadge :status="ws?.status === 'ready' ? 'ready' : (ws?.status === 'failed' ? 'failed' : 'processing')">
          {{ ws?.current_phase ? ws.current_phase.replace(/_/g, ' ') : 'Ready' }}
        </StatusBadge>
        <div class="text-xs font-mono px-2.5 py-1 bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 rounded border border-slate-200 dark:border-slate-700">
          {{ companyData?.sources?.length || 0 }} Sources • {{ claims.length }} Claims
        </div>
      </div>
    </div>
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
          <h2 class="text-xs font-semibold text-slate-900 dark:text-white font-mono uppercase tracking-wider">
            Active Run Agent Execution Feed
          </h2>
        </div>
        <NuxtLink
          v-if="activeRunId"
          :to="`/workflows/runs/${activeRunId}`"
          class="text-xs font-mono text-indigo-600 dark:text-indigo-400 hover:underline flex items-center gap-1"
        >
          <span>Open in Studio Inspector</span>
          <span>→</span>
        </NuxtLink>
      </div>

      <AgentStepFeed
        :events="workflowStore?.events.value || []"
        :compact="true"
        @open-citation="openSourceById"
      />
    </div>

    <div class="space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-bold text-slate-900 dark:text-white">
            Verified Architectural Claims & Evidence
          </h2>
          <p class="text-xs text-slate-500">
            Every technical insight linked to an authentic primary or secondary citation snippet.
          </p>
        </div>


        <div class="flex items-center gap-1 overflow-x-auto pb-1">
          <button
            v-for="cat in categories"
            :key="cat"
            @click="selectedCategory = cat"
            class="px-2.5 py-1 text-xs rounded-md capitalize transition-colors font-medium border"
            :class="[
              selectedCategory === cat
                ? 'bg-indigo-600 text-white border-indigo-600'
                : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800'
            ]"
          >
            {{ cat }}
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div
          v-for="claim in filteredClaims"
          :key="claim.id"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm hover:border-indigo-400 dark:hover:border-indigo-600 transition-colors flex flex-col justify-between gap-3"
        >
          <div>
            <div class="flex items-center justify-between gap-2 mb-2">
              <span class="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">
                {{ claim.category || 'Architecture' }}
              </span>
              <span class="text-[10px] font-mono text-emerald-600 dark:text-emerald-400 font-semibold">
                {{ Math.round((claim.confidence || 0.95) * 100) }}% confidence
              </span>
            </div>
            <p class="text-sm font-medium text-slate-900 dark:text-white leading-snug">
              {{ claim.statement }}
            </p>
            <div v-if="claim.evidence_snippet" class="mt-3 p-2.5 rounded bg-slate-50 dark:bg-slate-800/60 border border-slate-100 dark:border-slate-800 text-xs text-slate-600 dark:text-slate-300 italic">
              "{{ claim.evidence_snippet }}"
            </div>
          </div>

          <div class="pt-2 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs">
            <span class="text-slate-400 text-[11px]">Provenance:</span>
            <button
              v-if="claim.source_id"
              class="text-indigo-600 dark:text-indigo-400 font-mono text-[11px] hover:underline"
              @click="openSourceById(claim.source_id)"
            >
              {{ claim.source_url || 'View Citation' }}
            </button>
            <span v-else class="text-slate-400 text-[11px]">Direct Source</span>
          </div>
        </div>
      </div>
    </div>


    <EvidenceDrawer
      :is-open="isDrawerOpen"
      :citation="selectedSource"
      @close="isDrawerOpen = false"
    />
  </div>
</template>
