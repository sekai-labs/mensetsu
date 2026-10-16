<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { SourceCitation } from '~/features/interviews/models/interview.model'
import EvidenceDrawer from '~/components/ui/EvidenceDrawer.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceCompany } = useInterviewService()

const { data: companyData, pending, error: companyError, refresh: refreshSources } = await useAsyncData(
  `sources-index-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)

const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)
const { askWorkspaceKnowledgeQa } = useInterviewService()

const qaQuery = ref('')
const isAsking = ref(false)
const qaResponse = ref<{
  query: string
  answer: string
  cited_sources: SourceCitation[]
  cited_claims: Record<string, unknown>[]
  has_sufficient_evidence: boolean
} | null>(null)

const handleAskQa = async () => {
  if (!qaQuery.value.trim()) return
  isAsking.value = true
  try {
    qaResponse.value = await askWorkspaceKnowledgeQa(workspaceId.value, qaQuery.value.trim())
  } catch (err: unknown) {
    qaResponse.value = {
      query: qaQuery.value,
      answer: 'Failed to query knowledge base.',
      cited_sources: [],
      cited_claims: [],
      has_sufficient_evidence: false,
    }
  } finally {
    isAsking.value = false
  }
}

const openCitation = (source: SourceCitation) => {
  selectedSource.value = source
  isDrawerOpen.value = true
}
</script>

<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>{{ $t('sources.badge') }}</span>
          <span>•</span>
          <span>CITATION PROVENANCE</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('sources.title') }}
        </h1>
      </div>

      <div class="text-xs font-mono text-slate-500">
        {{ $t('sources.indexedCount', { count: companyData?.sources?.length || 0 }) }}
      </div>
    </div>


    <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg space-y-3 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="text-xs font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
          {{ $t('sources.qaTitle') }}
        </div>
        <span class="text-[10px] font-mono text-indigo-600 dark:text-indigo-400">{{ $t('sources.qaBadge') }}</span>
      </div>
      <div class="flex gap-2">
        <input
          v-model="qaQuery"
          type="text"
          :placeholder="$t('sources.qaPlaceholder')"
          class="flex-1 text-xs px-3 py-2 rounded border border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-850 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 font-mono"
          @keydown.enter="handleAskQa"
        />
        <button
          type="button"
          class="px-4 py-2 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors shrink-0"
          :disabled="isAsking || !qaQuery.trim()"
          @click="handleAskQa"
        >
          {{ isAsking ? $t('sources.askingButton') : $t('sources.askButton') }}
        </button>
      </div>
      <div v-if="qaResponse" class="p-3 bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-800 rounded text-xs space-y-2">
        <div class="flex items-center justify-between text-[11px] font-mono text-slate-400">
          <span>QUESTION: {{ qaResponse.query }}</span>
          <span :class="qaResponse.has_sufficient_evidence ? 'text-emerald-500 font-bold' : 'text-amber-500 font-bold'">
            {{ qaResponse.has_sufficient_evidence ? $t('sources.evidenceVerified') : $t('sources.noEvidence') }}
          </span>
        </div>
        <div class="text-slate-800 dark:text-slate-200 whitespace-pre-wrap leading-relaxed">
          {{ qaResponse.answer }}
        </div>
      </div>
    </div>

    <div v-if="companyError" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load verified sources</div>
      <p class="text-rose-600 dark:text-rose-400">{{ companyError.message || String(companyError) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshSources()"
      >
        Retry Sources Fetch
      </button>
    </div>

    <div v-else-if="pending" class="space-y-3">
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <div v-else-if="!companyData?.sources?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No sources indexed yet.
    </div>
    <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
      <div
        v-for="src in companyData.sources"
        :key="src.id"
        class="p-4 space-y-2 hover:bg-slate-50 dark:hover:bg-slate-850 cursor-pointer transition-colors"
        @click="openCitation(src)"
      >
        <div class="flex items-center justify-between gap-4">
          <span class="text-xs font-semibold text-slate-900 dark:text-white leading-snug">
            {{ src.title }}
          </span>
          <span class="text-[10px] font-mono text-emerald-600 dark:text-emerald-400 font-bold shrink-0">
            {{ Math.round((src.confidence || 1) * 100) }}% CONFIDENCE
          </span>
        </div>

        <blockquote class="p-3 bg-slate-50 dark:bg-slate-850 rounded border-l-2 border-indigo-500 text-xs text-slate-700 dark:text-slate-300 italic line-clamp-2 m-0">
          "{{ src.snippet }}"
        </blockquote>

        <div class="flex items-center justify-between text-[11px] font-mono text-slate-400 pt-1">
          <span class="truncate max-w-md text-indigo-600 dark:text-indigo-400">{{ src.source_url }}</span>
          <span>{{ src.retrieved_at ? new Date(src.retrieved_at).toLocaleDateString() : 'Recent' }}</span>
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
