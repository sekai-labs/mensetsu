<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { SourceCitation } from '~/features/interviews/models/interview.model'
import EvidenceDrawer from '~/components/ui/EvidenceDrawer.vue'
import { CheckCircle2, AlertCircle, Sparkles, AlertTriangle, Search, Filter } from 'lucide-vue-next'

definePageMeta({
  layout: 'workspace',
})

const { t } = useI18n()
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

const activeFilter = ref<'all' | 'strengths' | 'gaps' | 'wording' | 'risks'>('all')
const searchQuery = ref<string>('')
const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)

const openFirstCitation = () => {
  if (company.value?.sources?.length) {
    selectedSource.value = company.value.sources[0] || null
    isDrawerOpen.value = true
  }
}

const matchRatio = computed(() => {
  return match.value?.overall_match_ratio ? Math.round(match.value.overall_match_ratio * 100) : 0
})

const filteredStrengths = computed(() => {
  const items = match.value?.strengths || []
  if (!searchQuery.value.trim()) return items
  const q = searchQuery.value.toLowerCase()
  return items.filter(
    (s) =>
      s.skill?.toLowerCase().includes(q) ||
      s.evidence_quote?.toLowerCase().includes(q) ||
      s.relevance?.toLowerCase().includes(q)
  )
})

const filteredGaps = computed(() => {
  const items = match.value?.gaps || []
  if (!searchQuery.value.trim()) return items
  const q = searchQuery.value.toLowerCase()
  return items.filter(
    (g) =>
      g.skill?.toLowerCase().includes(q) ||
      g.requirement_quote?.toLowerCase().includes(q) ||
      g.recommended_focus?.toLowerCase().includes(q)
  )
})

const filteredWording = computed(() => {
  const items = match.value?.wording_improvements || []
  if (!searchQuery.value.trim()) return items
  const q = searchQuery.value.toLowerCase()
  return items.filter(
    (w) =>
      w.original_bullet?.toLowerCase().includes(q) ||
      w.improved_bullet?.toLowerCase().includes(q) ||
      w.rationale?.toLowerCase().includes(q)
  )
})

const filteredRisks = computed(() => {
  const items = match.value?.risks || []
  if (!searchQuery.value.trim()) return items
  const q = searchQuery.value.toLowerCase()
  return items.filter((r) => r.toLowerCase().includes(q))
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>{{ $t('match.badge') }}</span>
          <span>•</span>
          <span>EVIDENCE DECOMPOSITION</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('match.title') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          {{ $t('match.subtitle') }}
        </p>
      </div>

      <div class="flex items-center gap-3">
        <div class="text-right">
          <div class="text-[10px] font-mono uppercase text-slate-400">{{ $t('overview.matchAlignment') }}</div>
          <div
            class="text-2xl font-bold font-mono tabular-nums"
            :class="matchRatio >= 70 ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'"
          >
            {{ matchRatio }}%
          </div>
        </div>
      </div>
    </div>

    <!-- Error State -->
    <div v-if="errorMatch" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load match analysis</div>
      <p class="text-rose-600 dark:text-rose-400">{{ errorMatch.message || String(errorMatch) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshMatch()"
      >
        {{ $t('common.retry') }}
      </button>
    </div>

    <!-- Loading Skeleton -->
    <div v-else-if="pendingMatch" class="space-y-4">
      <div class="h-10 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-32 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <template v-else>
      <!-- Filters and Search Toolbar -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="flex items-center gap-1.5 overflow-x-auto pb-1 text-xs">
          <button
            type="button"
            @click="activeFilter = 'all'"
            class="px-3 py-1.5 rounded-md font-medium transition-colors"
            :class="activeFilter === 'all' ? 'bg-indigo-600 text-white shadow-2xs font-semibold' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'"
          >
            {{ $t('match.filters.all') }}
          </button>
          <button
            type="button"
            @click="activeFilter = 'strengths'"
            class="px-3 py-1.5 rounded-md font-medium transition-colors flex items-center gap-1.5"
            :class="activeFilter === 'strengths' ? 'bg-indigo-600 text-white shadow-2xs font-semibold' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'"
          >
            <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
            <span>{{ $t('match.filters.strengths') }} ({{ match?.strengths?.length || 0 }})</span>
          </button>
          <button
            type="button"
            @click="activeFilter = 'gaps'"
            class="px-3 py-1.5 rounded-md font-medium transition-colors flex items-center gap-1.5"
            :class="activeFilter === 'gaps' ? 'bg-indigo-600 text-white shadow-2xs font-semibold' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'"
          >
            <AlertCircle class="w-3.5 h-3.5 text-amber-500" />
            <span>{{ $t('match.filters.gaps') }} ({{ match?.gaps?.length || 0 }})</span>
          </button>
          <button
            type="button"
            @click="activeFilter = 'wording'"
            class="px-3 py-1.5 rounded-md font-medium transition-colors flex items-center gap-1.5"
            :class="activeFilter === 'wording' ? 'bg-indigo-600 text-white shadow-2xs font-semibold' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'"
          >
            <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
            <span>{{ $t('match.filters.wording') }} ({{ match?.wording_improvements?.length || 0 }})</span>
          </button>
          <button
            type="button"
            @click="activeFilter = 'risks'"
            class="px-3 py-1.5 rounded-md font-medium transition-colors flex items-center gap-1.5"
            :class="activeFilter === 'risks' ? 'bg-indigo-600 text-white shadow-2xs font-semibold' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'"
          >
            <AlertTriangle class="w-3.5 h-3.5 text-rose-500" />
            <span>{{ $t('match.filters.risks') }} ({{ match?.risks?.length || 0 }})</span>
          </button>
        </div>

        <div class="relative w-full sm:w-64">
          <Search class="w-3.5 h-3.5 absolute left-3 top-2.5 text-slate-400" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="$t('match.searchPlaceholder')"
            class="w-full pl-9 pr-3 py-1.5 text-xs rounded-md bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <!-- Strengths Matrix Section -->
      <div v-show="activeFilter === 'all' || activeFilter === 'strengths'" class="space-y-3">
        <div class="flex items-center justify-between">
          <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
            <CheckCircle2 class="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
            <span>{{ $t('match.verifiedMatchesTitle') }}</span>
          </h2>
          <span class="text-[11px] font-mono text-slate-400">{{ filteredStrengths.length }} verified items</span>
        </div>

        <div v-if="!filteredStrengths.length" class="p-6 text-center border border-dashed border-slate-200 dark:border-slate-800 rounded-lg text-xs text-slate-400">
          No matching verified strengths found.
        </div>

        <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-2xs">
          <div
            v-for="(st, idx) in filteredStrengths"
            :key="idx"
            class="p-4 space-y-2.5 hover:bg-slate-50/60 dark:hover:bg-slate-850/40 transition-colors"
          >
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 shrink-0"></span>
                <span class="text-xs font-bold text-slate-900 dark:text-white font-mono">{{ st.skill }}</span>
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800">
                  Verified Match
                </span>
              </div>
              <button
                type="button"
                class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium shrink-0"
                @click="openFirstCitation"
              >
                Inspect Citation →
              </button>
            </div>

            <div class="p-3 bg-slate-50 dark:bg-slate-850/70 rounded-md border-l-2 border-emerald-500 text-xs text-slate-700 dark:text-slate-300">
              <div class="text-[10px] font-mono uppercase text-slate-400 mb-0.5">{{ $t('match.resumeEvidence') }}</div>
              <p class="italic m-0">"{{ st.evidence_quote }}"</p>
            </div>

            <div class="text-[11px] text-slate-500">
              <span class="font-medium text-slate-700 dark:text-slate-300">{{ $t('match.roleRelevance') }}:</span> {{ st.relevance }}
            </div>
          </div>
        </div>
      </div>

      <!-- Identified Gaps Section -->
      <div v-show="activeFilter === 'all' || activeFilter === 'gaps'" class="space-y-3">
        <div class="flex items-center justify-between">
          <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
            <AlertCircle class="w-4 h-4 text-amber-600 dark:text-amber-400" />
            <span>{{ $t('match.gapsTitle') }}</span>
          </h2>
          <span class="text-[11px] font-mono text-slate-400">{{ filteredGaps.length }} items</span>
        </div>

        <div v-if="!filteredGaps.length" class="p-6 text-center border border-dashed border-slate-200 dark:border-slate-800 rounded-lg text-xs text-slate-400">
          No missing requirements or gaps found.
        </div>

        <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-2xs">
          <div
            v-for="(gap, idx) in filteredGaps"
            :key="idx"
            class="p-4 space-y-2.5 hover:bg-slate-50/60 dark:hover:bg-slate-850/40 transition-colors"
          >
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-amber-500 shrink-0"></span>
                <span class="text-xs font-bold text-slate-900 dark:text-white font-mono">{{ gap.skill }}</span>
                <span
                  class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded font-semibold"
                  :class="gap.importance === 'required' ? 'bg-rose-50 text-rose-700 dark:bg-rose-950/60 dark:text-rose-300 border border-rose-200 dark:border-rose-900' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'"
                >
                  {{ gap.importance }}
                </span>
              </div>
              <NuxtLink
                :to="`/interviews/${workspaceId}/learning`"
                class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium shrink-0"
              >
                Study Plan Module →
              </NuxtLink>
            </div>

            <div class="p-3 bg-slate-50 dark:bg-slate-850/70 rounded-md border-l-2 border-amber-500 text-xs text-slate-700 dark:text-slate-300">
              <div class="text-[10px] font-mono uppercase text-slate-400 mb-0.5">{{ $t('match.jobRequirement') }}</div>
              <p class="m-0">"{{ gap.requirement_quote }}"</p>
            </div>

            <div class="text-xs text-amber-900 dark:text-amber-200 bg-amber-50/60 dark:bg-amber-950/30 p-2.5 rounded border border-amber-200/60 dark:border-amber-900/40">
              <span class="font-semibold">{{ $t('match.preparationStrategy') }}:</span> {{ gap.recommended_focus }}
            </div>
          </div>
        </div>
      </div>

      <!-- Resume Wording Improvements -->
      <div v-show="activeFilter === 'all' || activeFilter === 'wording'" class="space-y-3">
        <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
          <Sparkles class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          <span>{{ $t('match.wordingTitle') }}</span>
        </h2>

        <div v-if="!filteredWording.length" class="p-6 text-center border border-dashed border-slate-200 dark:border-slate-800 rounded-lg text-xs text-slate-400">
          No specific bullet improvements suggested.
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(w, idx) in filteredWording"
            :key="idx"
            class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs space-y-2.5"
          >
            <div class="space-y-1">
              <span class="text-[10px] font-mono uppercase text-slate-400">{{ $t('match.originalBullet') }}</span>
              <p class="text-xs text-slate-500 line-through m-0">
                {{ w.original_bullet }}
              </p>
            </div>

            <div class="space-y-1">
              <span class="text-[10px] font-mono uppercase text-emerald-600 dark:text-emerald-400 font-bold">{{ $t('match.suggestedBullet') }}</span>
              <p class="text-xs font-semibold text-slate-900 dark:text-white m-0">
                {{ w.improved_bullet }}
              </p>
            </div>

            <div class="text-[11px] text-slate-500 pt-2 border-t border-slate-100 dark:border-slate-800">
              <span class="font-medium text-slate-700 dark:text-slate-300">{{ $t('match.rationale') }}:</span> {{ w.rationale }}
            </div>
          </div>
        </div>
      </div>

      <!-- Interview Risks -->
      <div v-show="activeFilter === 'all' || activeFilter === 'risks'" class="space-y-3">
        <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
          <AlertTriangle class="w-4 h-4 text-rose-500" />
          <span>{{ $t('match.risksTitle') }}</span>
        </h2>

        <div v-if="!filteredRisks.length" class="p-6 text-center border border-dashed border-slate-200 dark:border-slate-800 rounded-lg text-xs text-slate-400">
          No critical probe risks identified.
        </div>

        <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-2xs">
          <div
            v-for="(risk, idx) in filteredRisks"
            :key="idx"
            class="p-3.5 flex items-start gap-3 hover:bg-slate-50/60 dark:hover:bg-slate-850/40 transition-colors"
          >
            <span class="w-2 h-2 rounded-full bg-rose-500 mt-1 shrink-0"></span>
            <p class="text-xs text-slate-700 dark:text-slate-300 m-0 leading-relaxed">
              {{ risk }}
            </p>
          </div>
        </div>
      </div>
    </template>

    <!-- Source Evidence Drawer -->
    <EvidenceDrawer
      :is-open="isDrawerOpen"
      :citation="selectedSource"
      @close="isDrawerOpen = false"
    />
  </div>
</template>
