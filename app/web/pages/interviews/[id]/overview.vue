<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace, getWorkspaceMatch, getWorkspaceCompany } = useInterviewService()

const { data: ws, pending: pendingWs, error: errorWs, refresh: refreshWs } = await useAsyncData(`ws-ov-${workspaceId.value}`, () => getWorkspace(workspaceId.value))
const { data: match } = await useAsyncData(`ws-ov-match-${workspaceId.value}`, () => getWorkspaceMatch(workspaceId.value))
const { data: company } = await useAsyncData(`ws-ov-comp-${workspaceId.value}`, () => getWorkspaceCompany(workspaceId.value))
</script>
<template>
  <div class="space-y-8">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>WORKSPACE DOSSIER</span>
          <span>•</span>
          <span class="text-slate-400 font-mono">ID: {{ workspaceId }}</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Overview & Intelligence Status
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          Target role analysis, match alignment, and preparation roadmap for {{ ws?.company_name || 'Organization' }}.
        </p>
      </div>

      <div class="flex items-center gap-2">
        <NuxtLink :to="`/interviews/${workspaceId}/company`">
          <AppButton size="sm">
            Read Technical Dossier →
          </AppButton>
        </NuxtLink>
      </div>
    </div>

    <div v-if="errorWs" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load workspace overview</div>
      <p class="text-rose-600 dark:text-rose-400">{{ errorWs.message || String(errorWs) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshWs()"
      >
        Retry Overview Fetch
      </button>
    </div>

    <div v-else-if="pendingWs" class="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-24 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">Match Alignment</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-emerald-600 dark:text-emerald-400 mt-1">
          {{ match?.overall_match_ratio ? Math.round(match.overall_match_ratio * 100) : 0 }}%
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          {{ match?.strengths?.length || 0 }} verified strengths, {{ match?.gaps?.length || 0 }} gap areas
        </p>
      </div>

      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">Pipeline State</div>
        <div class="text-sm font-semibold text-slate-900 dark:text-white mt-1 capitalize">
          {{ ws?.current_phase ? ws.current_phase.replace(/_/g, ' ') : 'Pending' }}
        </div>
        <p class="text-[11px] text-slate-500 font-mono mt-1">
          Progress: {{ ws?.progress_percent || 0 }}%
        </p>
      </div>

      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">Verified Sources</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-indigo-600 dark:text-indigo-400 mt-1">
          {{ company?.sources?.length || 0 }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          Indexed engineering publications & RFCs
        </p>
      </div>
    </div>


    <div class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Executive Company Profile
      </h2>
      <div class="p-5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg text-xs leading-relaxed text-slate-700 dark:text-slate-300 space-y-3">
        <p>
          {{ company?.overview || 'Dossier research underway. Background workers are synthesizing whitepapers, engineering architecture posts, and candidate qualifications.' }}
        </p>
        <div v-if="company?.business_model" class="pt-2 border-t border-slate-100 dark:border-slate-800">
          <span class="font-semibold text-slate-900 dark:text-white">Business Model & Monetization: </span>
          <span>{{ company.business_model }}</span>
        </div>
      </div>
    </div>


    <div class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Workspace Preparation Sections
      </h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
        <NuxtLink
          :to="`/interviews/${workspaceId}/match`"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1"
        >
          <div class="font-semibold text-slate-900 dark:text-white">Match Analysis & Evidence</div>
          <p class="text-slate-500">Review matched skills, detected gaps, and targeted resume wording improvements.</p>
        </NuxtLink>

        <NuxtLink
          :to="`/interviews/${workspaceId}/company`"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1"
        >
          <div class="font-semibold text-slate-900 dark:text-white">Technical Dossier Reader</div>
          <p class="text-slate-500">6-chapter architectural dossier, infrastructure topology, and interview rubrics.</p>
        </NuxtLink>

        <NuxtLink
          :to="`/interviews/${workspaceId}/knowledge`"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1"
        >
          <div class="font-semibold text-slate-900 dark:text-white">Knowledge Graph</div>
          <p class="text-slate-500">Interactive topological diagram mapping candidate mastery against company stack.</p>
        </NuxtLink>

        <NuxtLink
          :to="`/interviews/${workspaceId}/questions`"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1"
        >
          <div class="font-semibold text-slate-900 dark:text-white">Question Book & Practice</div>
          <p class="text-slate-500">Contextual interview questions with outline answers, evaluation rubrics, and scratchpad.</p>
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
