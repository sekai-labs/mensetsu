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
const { getWorkspaceMatch, getWorkspaceCompany } = useInterviewService()

const { data: match, pending: pendingMatch, error: errorMatch, refresh: refreshMatch } = await useAsyncData(
  `match-analysis-${workspaceId.value}`,
  () => getWorkspaceMatch(workspaceId.value)
)
const { data: company } = await useAsyncData(
  `match-company-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)

const isScoreEvidenceOpen = ref<boolean>(false)
const activeTab = ref<string>('strengths')
const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)


const openFirstCitation = () => {
  if (company.value?.sources?.length) {
    selectedSource.value = company.value.sources[0] || null
    isDrawerOpen.value = true
  }
}
</script>

<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>MATCH ANALYSIS</span>
          <span>•</span>
          <span>EXPLAINABLE ALIGNMENT</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          CV / JD Match Analysis & Evidence Decomposition
        </h1>
      </div>

      <div class="flex items-center gap-3">
        <div class="text-right">
          <div class="text-[10px] font-mono uppercase text-slate-400">Overall Ratio</div>
          <button
            type="button"
            class="text-2xl font-bold font-mono text-emerald-600 dark:text-emerald-400 tabular-nums hover:underline flex items-center gap-1 cursor-pointer"
            title="Click to view score breakdown and underlying evidence"
            @click="isScoreEvidenceOpen = !isScoreEvidenceOpen"
          >
            <span>{{ match?.overall_match_ratio ? Math.round(match.overall_match_ratio * 100) : 0 }}%</span>
            <span class="text-xs text-slate-400 font-sans font-normal">🔍</span>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isScoreEvidenceOpen" class="p-4 bg-emerald-50/40 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-800 rounded-lg text-xs space-y-2">
      <div class="flex items-center justify-between">
        <span class="font-bold text-emerald-800 dark:text-emerald-300 font-mono text-[11px] uppercase tracking-wider">
          Score Evidence & Alignment Breakdown
        </span>
        <button type="button" class="text-slate-400 hover:text-slate-600 text-xs font-mono" @click="isScoreEvidenceOpen = false">✕</button>
      </div>
      <p class="text-slate-700 dark:text-slate-300 leading-relaxed">
        Score derived from {{ match?.strengths?.length || 0 }} verified skill strengths against {{ match?.gaps?.length || 0 }} gap requirements.
        Confidence calculated autonomously based on indexed primary sources.
      </p>
      <div v-if="company?.sources?.length" class="pt-2 border-t border-emerald-100 dark:border-emerald-900/60 flex items-center justify-between text-[11px]">
        <span class="text-slate-500">Supporting Grounded Corpus:</span>
        <button type="button" class="text-indigo-600 dark:text-indigo-400 hover:underline font-mono" @click="openFirstCitation">
          {{ company.sources[0]?.title || 'Inspect Root Primary Source' }} →
        </button>
      </div>
    </div>

    <div v-if="errorMatch" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load match analysis</div>
      <p class="text-rose-600 dark:text-rose-400">{{ errorMatch.message || String(errorMatch) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshMatch()"
      >
        Retry Analysis Fetch
      </button>
    </div>

    <div v-else-if="pendingMatch" class="space-y-4">
      <div class="h-6 bg-slate-200 dark:bg-slate-800 rounded animate-pulse w-1/4"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <template v-else>
      <div class="border-b border-slate-200 dark:border-slate-800 flex items-center gap-2">
      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px flex items-center gap-1.5"
        :class="activeTab === 'strengths'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'strengths'"
      >
        <span>Verified Strengths</span>
        <span class="px-1.5 py-0.2 rounded-full bg-slate-100 dark:bg-slate-800 text-[10px] font-mono">
          {{ match?.strengths?.length || 0 }}
        </span>
      </button>

      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px flex items-center gap-1.5"
        :class="activeTab === 'gaps'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'gaps'"
      >
        <span>Identified Gaps</span>
        <span class="px-1.5 py-0.2 rounded-full bg-slate-100 dark:bg-slate-800 text-[10px] font-mono">
          {{ match?.gaps?.length || 0 }}
        </span>
      </button>

      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px flex items-center gap-1.5"
        :class="activeTab === 'wording'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'wording'"
      >
        <span>Wording Improvements</span>
        <span class="px-1.5 py-0.2 rounded-full bg-slate-100 dark:bg-slate-800 text-[10px] font-mono">
          {{ match?.wording_improvements?.length || 0 }}
        </span>
      </button>

      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px flex items-center gap-1.5"
        :class="activeTab === 'risks'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'risks'"
      >
        <span>Interview Risks</span>
        <span class="px-1.5 py-0.2 rounded-full bg-slate-100 dark:bg-slate-800 text-[10px] font-mono">
          {{ match?.risks?.length || 0 }}
        </span>
      </button>
    </div>


    <div v-show="activeTab === 'strengths'" class="space-y-4">
      <div v-if="!match?.strengths?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
        No verified strengths extracted yet.
      </div>

      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="(st, idx) in match.strengths"
          :key="idx"
          class="p-4 space-y-2 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
              <span class="text-xs font-semibold text-slate-900 dark:text-white font-mono">{{ st.skill }}</span>
            </div>
            <button
              type="button"
              class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium"
              @click="openFirstCitation"
            >
              Inspect Evidence →
            </button>
          </div>

          <blockquote class="p-3 bg-slate-50 dark:bg-slate-850 rounded border-l-2 border-emerald-500 text-xs text-slate-700 dark:text-slate-300 italic">
            "{{ st.evidence_quote }}"
          </blockquote>

          <div class="text-[11px] text-slate-500">
            <span class="font-medium text-slate-600 dark:text-slate-400">Relevance:</span> {{ st.relevance }}
          </div>
        </div>
      </div>
    </div>


    <div v-show="activeTab === 'gaps'" class="space-y-4">
      <div v-if="!match?.gaps?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
        No missing requirements or gaps identified.
      </div>

      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="(gap, idx) in match.gaps"
          :key="idx"
          class="p-4 space-y-2 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-amber-500"></span>
              <span class="text-xs font-semibold text-slate-900 dark:text-white font-mono">{{ gap.skill }}</span>
              <span
                class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded font-bold"
                :class="gap.importance === 'required' ? 'bg-rose-50 text-rose-700 dark:bg-rose-950 dark:text-rose-300' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'"
              >
                {{ gap.importance }}
              </span>
            </div>
            <div class="flex items-center gap-3">
              <button
                type="button"
                class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium"
                @click="openFirstCitation"
              >
                Source Citation →
              </button>
              <NuxtLink
                :to="`/interviews/${workspaceId}/learning`"
                class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium"
              >
                Learning Plan →
              </NuxtLink>
            </div>
          </div>
          <div class="text-xs text-slate-700 dark:text-slate-300">
            <span class="font-medium">Requirement in JD:</span> "{{ gap.requirement_quote }}"
          </div>

          <div class="text-xs text-slate-600 dark:text-slate-400 bg-amber-50/50 dark:bg-amber-950/20 p-2.5 rounded border border-amber-200/60 dark:border-amber-900/40">
            <span class="font-medium text-amber-900 dark:text-amber-200">Recommended Preparation:</span> {{ gap.recommended_focus }}
          </div>
        </div>
      </div>
    </div>

    <div v-show="activeTab === 'wording'" class="space-y-4">
      <div v-if="!match?.wording_improvements?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
        No specific resume bullet rewrites suggested.
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="(w, idx) in match.wording_improvements"
          :key="idx"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm space-y-3"
        >
          <div class="space-y-1">
            <span class="text-[10px] font-mono uppercase text-slate-400">Current Resume Bullet</span>
            <p class="text-xs text-slate-600 dark:text-slate-400 line-through m-0">
              {{ w.original_bullet }}
            </p>
          </div>

          <div class="space-y-1">
            <span class="text-[10px] font-mono uppercase text-emerald-600 dark:text-emerald-400 font-bold">Recommended Precision Bullet</span>
            <p class="text-xs font-medium text-slate-900 dark:text-white m-0">
              {{ w.improved_bullet }}
            </p>
          </div>
          <div class="text-[11px] text-slate-500 pt-2 border-t border-slate-100 dark:border-slate-800">
            <span class="font-medium text-slate-600 dark:text-slate-400">Rationale:</span> {{ w.rationale }}
          </div>
        </div>
      </div>
    </div>


    <div v-show="activeTab === 'risks'" class="space-y-4">
      <div v-if="!match?.risks?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
        No critical probe risks identified.
      </div>

      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="(risk, idx) in match.risks"
          :key="idx"
          class="p-4 flex items-start gap-3 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <span class="w-2 h-2 rounded-full bg-rose-500 mt-1.5 shrink-0"></span>
          <p class="text-xs text-slate-700 dark:text-slate-300 m-0 leading-relaxed">
            {{ risk }}
          </p>
        </div>
      </div>
    </div>
    </template>


    <EvidenceDrawer
      :is-open="isDrawerOpen"
      :citation="selectedSource"
      @close="isDrawerOpen = false"
    />
  </div>
</template>
