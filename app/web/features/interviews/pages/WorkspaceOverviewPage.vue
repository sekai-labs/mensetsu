<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { formatPercentage } from '~/utils/formatters'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace, getWorkspaceMatch, getWorkspaceCompany } = useInterviewService()

const { data: workspace } = await useAsyncData(
  `overview-ws-${workspaceId.value}`,
  () => getWorkspace(workspaceId.value)
)
const { data: matchData } = await useAsyncData(
  `overview-match-${workspaceId.value}`,
  () => getWorkspaceMatch(workspaceId.value)
)
const { data: companyData } = await useAsyncData(
  `overview-comp-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
      <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
        Preparation Overview
      </h1>
      <p class="text-xs text-slate-500 mt-1 leading-relaxed">
        Synthesis of candidate experience versus {{ workspace?.company_name }}'s requirements. Review identified gaps, explore lessons, and drill curated technical questions.
      </p>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-5">
      <NuxtLink
        :to="`/interviews/${workspaceId}/match`"
        class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm hover:border-indigo-400 transition"
      >
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Candidate Match</span>
        <div class="text-3xl font-bold font-mono text-slate-900 dark:text-white mt-1">
          {{ formatPercentage(matchData?.overall_match_ratio) }}%
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Grounded against verified requirements →</p>
      </NuxtLink>

      <NuxtLink
        :to="`/interviews/${workspaceId}/skills`"
        class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm hover:border-amber-400 transition"
      >
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Skill Gaps to Bridge</span>
        <div class="text-3xl font-bold font-mono text-amber-600 dark:text-amber-400 mt-1">
          {{ matchData?.gaps?.length ?? 0 }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Inspect missing evidence →</p>
      </NuxtLink>

      <NuxtLink
        :to="`/interviews/${workspaceId}/questions`"
        class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm hover:border-indigo-400 transition"
      >
        <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Curated Rubrics</span>
        <div class="text-3xl font-bold font-mono text-indigo-600 dark:text-indigo-400 mt-1">
          Ready
        </div>
        <p class="text-[11px] text-slate-500 mt-1">Practice interview rubrics →</p>
      </NuxtLink>
    </div>

    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Target Company Context</h2>
        <NuxtLink :to="`/interviews/${workspaceId}/company`" class="text-xs font-semibold text-indigo-600 hover:text-indigo-500">
          Full company report →
        </NuxtLink>
      </div>

      <p class="text-xs text-slate-600 dark:text-slate-300 leading-relaxed">
        {{ companyData?.overview }}
      </p>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-slate-100 dark:border-slate-800 text-xs">
        <div>
          <span class="font-semibold text-slate-700 dark:text-slate-300">Engineering Environment:</span>
          <div class="flex flex-wrap gap-1.5 mt-1.5">
            <span
              v-for="item in companyData?.technology_stack?.slice(0, 5)"
              :key="item"
              class="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 font-mono text-[11px] text-slate-700 dark:text-slate-300"
            >
              {{ item }}
            </span>
          </div>
        </div>

        <div>
          <span class="font-semibold text-slate-700 dark:text-slate-300">Business Model:</span>
          <p class="text-slate-500 mt-1">{{ companyData?.business_model }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
