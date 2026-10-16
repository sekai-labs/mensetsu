<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import AppButton from '~/components/ui/AppButton.vue'
import StatusBadge from '~/components/ui/StatusBadge.vue'
import {
  CheckCircle2,
  AlertCircle,
  Building2,
  BookOpen,
  ArrowRight,
  ExternalLink,
  Layers,
  HelpCircle,
  Sparkles,
} from 'lucide-vue-next'

definePageMeta({
  layout: 'workspace',
})

const { t } = useI18n()
const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace, getWorkspaceMatch, getWorkspaceCompany, updateWorkspaceObjective } = useInterviewService()

const { data: ws, pending: pendingWs, error: errorWs, refresh: refreshWs } = await useAsyncData(`ws-ov-${workspaceId.value}`, () => getWorkspace(workspaceId.value))
const { data: match } = await useAsyncData(`ws-ov-match-${workspaceId.value}`, () => getWorkspaceMatch(workspaceId.value))
const { data: company } = await useAsyncData(`ws-ov-comp-${workspaceId.value}`, () => getWorkspaceCompany(workspaceId.value))

const isUpdatingObjective = ref(false)
const showObjectiveModal = ref(false)
const objectivePrompt = ref('')
const objectiveSuccess = ref(false)
const objectiveError = ref('')

const matchRatio = computed(() => {
  return match.value?.overall_match_ratio ? Math.round(match.value.overall_match_ratio * 100) : 0
})

const handleUpdateObjective = async () => {
  if (!objectivePrompt.value.trim()) return
  isUpdatingObjective.value = true
  objectiveError.value = ''
  objectiveSuccess.value = false
  try {
    await updateWorkspaceObjective(workspaceId.value, objectivePrompt.value.trim())
    objectiveSuccess.value = true
    objectivePrompt.value = ''
    await refreshWs()
    setTimeout(() => {
      showObjectiveModal.value = false
      objectiveSuccess.value = false
    }, 1500)
  } catch (err: any) {
    objectiveError.value = err?.data?.error?.message || err?.message || 'Failed to update research objective.'
  } finally {
    isUpdatingObjective.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>{{ $t('overview.badge') }}</span>
          <span>•</span>
          <span class="text-slate-400 font-mono">ID: {{ workspaceId }}</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('overview.title', { company: ws?.company_name || 'Target Role' }) }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
          {{ $t('overview.subtitle', { role: ws?.role_title || 'Role' }) }}
        </p>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          @click="showObjectiveModal = true"
          class="px-3 py-1.5 text-xs font-medium rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-850 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 transition flex items-center gap-1.5"
        >
          <span>{{ $t('overview.updateFocus') }}</span>
        </button>
        <NuxtLink :to="`/interviews/${workspaceId}/company`">
          <AppButton size="sm">
            {{ $t('overview.readDossier') }}
          </AppButton>
        </NuxtLink>
      </div>
    </div>

    <!-- Error State -->
    <div v-if="errorWs" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load workspace overview</div>
      <p class="text-rose-600 dark:text-rose-400">{{ errorWs.message || String(errorWs) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshWs()"
      >
        Retry Fetch
      </button>
    </div>

    <!-- Loading Skeleton -->
    <div v-else-if="pendingWs" class="grid grid-cols-2 sm:grid-cols-4 gap-4">
      <div v-for="i in 4" :key="i" class="h-20 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <!-- Metrics Bar -->
    <div v-else class="grid grid-cols-2 sm:grid-cols-4 gap-4">
      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('overview.matchAlignment') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight mt-1" :class="matchRatio >= 70 ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'">
          {{ matchRatio }}%
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          {{ match?.strengths?.length || 0 }} matches • {{ match?.gaps?.length || 0 }} gaps
        </p>
      </div>

      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('overview.targetSeniority') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-slate-900 dark:text-white mt-1 capitalize">
          {{ ws?.seniority || 'Senior' }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          Expected domain depth
        </p>
      </div>

      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('overview.prepTimeline') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-indigo-600 dark:text-indigo-400 mt-1">
          {{ ws?.prep_time_weeks || 2 }} wks
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          Target interview horizon
        </p>
      </div>

      <div class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs">
        <div class="text-[11px] font-mono uppercase tracking-wider text-slate-500">{{ $t('overview.indexedSources') }}</div>
        <div class="text-2xl font-bold font-mono tracking-tight text-slate-900 dark:text-white mt-1">
          {{ company?.sources?.length || 0 }}
        </div>
        <p class="text-[11px] text-slate-500 mt-1">
          Verified citations
        </p>
      </div>
    </div>

    <!-- 2-Column Main Layout -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Left Column: Key Matches & Gaps -->
      <div class="space-y-6">
        <!-- Matched Skills -->
        <div class="p-5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs space-y-3">
          <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
            <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
              <CheckCircle2 class="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
              <span>{{ $t('overview.verifiedMatches') }}</span>
            </h2>
            <NuxtLink :to="`/interviews/${workspaceId}/match`" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium">
              View All ({{ match?.strengths?.length || 0 }}) →
            </NuxtLink>
          </div>

          <div v-if="!match?.strengths?.length" class="text-xs text-slate-500 py-4 text-center">
            No verified matches identified yet.
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="(item, idx) in match.strengths.slice(0, 3)"
              :key="idx"
              class="p-3 rounded-md bg-emerald-50/50 dark:bg-emerald-950/20 border border-emerald-100 dark:border-emerald-900/40"
            >
              <div class="text-xs font-semibold text-emerald-900 dark:text-emerald-300">
                {{ item.skill }}
              </div>
              <div class="text-[11px] text-emerald-700/80 dark:text-emerald-400/80 mt-1 line-clamp-2">
                "{{ item.evidence_quote }}"
              </div>
            </div>
          </div>
        </div>

        <!-- Priority Gaps -->
        <div class="p-5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs space-y-3">
          <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
            <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
              <AlertCircle class="w-4 h-4 text-amber-600 dark:text-amber-400" />
              <span>{{ $t('overview.priorityGaps') }}</span>
            </h2>
            <NuxtLink :to="`/interviews/${workspaceId}/learning`" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium">
              Study Plan →
            </NuxtLink>
          </div>

          <div v-if="!match?.gaps?.length" class="text-xs text-slate-500 py-4 text-center">
            No critical skill gaps detected.
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="(gap, idx) in match.gaps.slice(0, 3)"
              :key="idx"
              class="p-3 rounded-md bg-amber-50/50 dark:bg-amber-950/20 border border-amber-100 dark:border-amber-900/40"
            >
              <div class="flex items-center justify-between gap-2">
                <div class="text-xs font-semibold text-amber-900 dark:text-amber-300">
                  {{ gap.skill }}
                </div>
                <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-amber-200/50 dark:bg-amber-900/50 text-amber-800 dark:text-amber-200">
                  {{ gap.importance || 'Required' }}
                </span>
              </div>
              <p class="text-[11px] text-amber-700/80 dark:text-amber-400/80 mt-1">
                {{ gap.recommended_focus || gap.requirement_quote || 'Address this requirement in study plan lessons.' }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column: Architecture & Recommended Actions -->
      <div class="space-y-6">
        <!-- Company Profile & Architecture Highlight -->
        <div class="p-5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-2xs space-y-3">
          <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
            <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-1.5">
              <Building2 class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
              <span>{{ $t('overview.architectureOverview') }}</span>
            </h2>
            <NuxtLink :to="`/interviews/${workspaceId}/company`" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium">
              {{ $t('overview.fullDossier') }}
            </NuxtLink>
          </div>

          <p class="text-xs leading-relaxed text-slate-600 dark:text-slate-300">
            {{ company?.overview || 'Dossier synthesis underway. Background processes are compiling architecture posts and whitepapers.' }}
          </p>

          <div v-if="company?.tech_stack?.length" class="pt-2 border-t border-slate-100 dark:border-slate-800">
            <div class="text-[10px] font-mono uppercase text-slate-400 mb-1.5">{{ $t('overview.productionStack') }}</div>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="tech in company.tech_stack.slice(0, 8)"
                :key="tech.name"
                class="px-2 py-0.5 text-xs font-mono rounded bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300"
              >
                {{ tech.name }}
              </span>
            </div>
          </div>
        </div>

        <!-- Quick Launch Preparation Cards -->
        <div class="grid grid-cols-2 gap-3">
          <NuxtLink
            :to="`/interviews/${workspaceId}/questions`"
            class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1 block shadow-2xs group"
          >
            <div class="flex items-center justify-between">
              <HelpCircle class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
              <ArrowRight class="w-3.5 h-3.5 text-slate-400 group-hover:text-indigo-600 transition-colors" />
            </div>
            <div class="font-semibold text-xs text-slate-900 dark:text-white pt-1">{{ $t('navigation.items.questions') }}</div>
            <p class="text-[11px] text-slate-500">{{ $t('overview.questionBankDesc') }}</p>
          </NuxtLink>

          <NuxtLink
            :to="`/interviews/${workspaceId}/practice`"
            class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-indigo-500 transition-colors space-y-1 block shadow-2xs group"
          >
            <div class="flex items-center justify-between">
              <Layers class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
              <ArrowRight class="w-3.5 h-3.5 text-slate-400 group-hover:text-indigo-600 transition-colors" />
            </div>
            <div class="font-semibold text-xs text-slate-900 dark:text-white pt-1">{{ $t('navigation.items.practice') }}</div>
            <p class="text-[11px] text-slate-500">{{ $t('overview.mockPracticeDesc') }}</p>
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>

  <!-- Objective Update Modal -->
  <div v-if="showObjectiveModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-2xs">
    <div class="w-full max-w-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl shadow-xl p-5 space-y-4">
      <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
        <div>
          <h3 class="text-sm font-bold text-slate-900 dark:text-white">{{ $t('overview.updateModalTitle') }}</h3>
          <p class="text-xs text-slate-500 mt-0.5">{{ $t('overview.updateModalDesc') }}</p>
        </div>
        <button @click="showObjectiveModal = false" class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 text-sm">✕</button>
      </div>

      <div v-if="objectiveError" class="p-3 bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 text-xs rounded-lg border border-rose-200 dark:border-rose-900">
        {{ objectiveError }}
      </div>
      <div v-if="objectiveSuccess" class="p-3 bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 text-xs rounded-lg border border-emerald-200 dark:border-emerald-900">
        Focus updated. Research workflow extended.
      </div>

      <div>
        <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">New Technical Focus / Query</label>
        <textarea
          v-model="objectivePrompt"
          rows="3"
          placeholder="e.g. Focus on their migration to ClickHouse or payment ledger settlement..."
          class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg p-2.5 text-xs text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
        ></textarea>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-slate-800">
        <button
          type="button"
          @click="showObjectiveModal = false"
          class="px-3 py-1.5 text-xs text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="button"
          @click="handleUpdateObjective"
          :disabled="!objectivePrompt.trim() || isUpdatingObjective"
          class="px-4 py-1.5 text-xs font-medium bg-indigo-600 text-white rounded-lg hover:bg-indigo-500 disabled:opacity-50 transition flex items-center gap-1.5"
        >
          <span v-if="isUpdatingObjective" class="animate-spin w-3 h-3 border-2 border-white border-t-transparent rounded-full"></span>
          <span>{{ isUpdatingObjective ? 'Dispatching...' : 'Update Focus' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
