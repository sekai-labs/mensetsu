<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { formatPercentage } from '~/utils/formatters'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceMatch } = useInterviewService()

const { data: matchData } = await useAsyncData(
  `match-page-${workspaceId.value}`,
  () => getWorkspaceMatch(workspaceId.value)
)
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">CV vs. JD Match Analysis</h1>
          <p class="text-xs text-slate-500 mt-1">
            Explainable breakdown showing how candidate achievements map to explicit role requirements.
          </p>
        </div>
        <div class="flex items-center sm:flex-col sm:items-end justify-between sm:justify-start pt-2 sm:pt-0 border-t sm:border-t-0 border-slate-100 dark:border-slate-800">
          <span class="text-2xl font-bold font-mono text-indigo-600 dark:text-indigo-400">
            {{ formatPercentage(matchData?.overall_match_ratio) }}%
          </span>
          <div class="text-[11px] text-slate-400">Overall Match</div>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
          <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Verified Strengths</h2>
          <span class="text-xs px-2 py-0.5 rounded bg-emerald-50 text-emerald-700 font-mono font-semibold">
            {{ matchData?.strengths?.length ?? 0 }} Evidenced
          </span>
        </div>

        <div v-if="!matchData?.strengths || matchData.strengths.length === 0" class="text-xs text-slate-500">
          No strengths extracted yet.
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(s, idx) in matchData.strengths"
            :key="idx"
            class="p-4 rounded-md border border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-850"
          >
            <div class="font-semibold text-slate-900 dark:text-white text-xs">{{ s.skill }}</div>
            <p class="text-xs text-slate-600 dark:text-slate-400 mt-1 italic">"{{ s.evidence_quote }}"</p>
            <div class="text-[11px] text-emerald-700 dark:text-emerald-400 mt-2 font-medium">{{ s.relevance }}</div>
          </div>
        </div>
      </div>

      <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
          <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Identified Gaps & Focus</h2>
          <span class="text-xs px-2 py-0.5 rounded bg-amber-50 text-amber-700 font-mono font-semibold">
            {{ matchData?.gaps?.length ?? 0 }} Gaps
          </span>
        </div>

        <div v-if="!matchData?.gaps || matchData.gaps.length === 0" class="text-xs text-slate-500">
          No gaps detected. High alignment found.
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(g, idx) in matchData.gaps"
            :key="idx"
            class="p-4 rounded-md border border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-850"
          >
            <div class="flex items-center justify-between">
              <span class="font-semibold text-slate-900 dark:text-white text-xs">{{ g.skill }}</span>
              <span
                class="text-[10px] font-mono uppercase px-2 py-0.5 rounded"
                :class="g.importance === 'mandatory' ? 'bg-rose-100 text-rose-700 dark:bg-rose-950/60 dark:text-rose-400' : 'bg-slate-200 text-slate-700 dark:bg-slate-800 dark:text-slate-300'"
              >
                {{ g.importance }}
              </span>
            </div>
            <p class="text-xs text-slate-600 dark:text-slate-400 mt-1">"{{ g.requirement_quote }}"</p>
            <div class="text-[11px] text-indigo-600 dark:text-indigo-400 mt-2 font-medium">
              Study: {{ g.recommended_focus }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="matchData?.wording_improvements && matchData.wording_improvements.length > 0" class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
      <h2 class="text-sm font-semibold text-slate-900 dark:text-white">CV Wording Precision Advice</h2>
      <div class="space-y-3">
        <div
          v-for="(w, idx) in matchData.wording_improvements"
          :key="idx"
          class="p-4 rounded-md border border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-850 space-y-2 text-xs"
        >
          <div class="text-slate-500 line-through">"{{ w.original_text }}"</div>
          <div class="text-emerald-700 dark:text-emerald-400 font-medium">→ "{{ w.suggested_revision }}"</div>
          <div class="text-[11px] text-slate-400">{{ w.rationale }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
