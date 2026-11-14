<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { safeExternalHttpUrl } from '~/utils/safe-external-url'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type {
  JobApplicationResponseDto,
  ApplicationStatus,
} from '~/features/interviews/models/interview.model'
import AppButton from '~/components/ui/AppButton.vue'
import {
  Briefcase,
  ExternalLink,
  CheckCircle2,
  AlertTriangle,
  Clock,
  HelpCircle,
  XCircle,
  ArrowRight,
  RefreshCw,
  Sparkles,
} from 'lucide-vue-next'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()
const {
  listApplications,
  reconcileApplication,
  cleanWorkspaceId,
} = useInterviewService()

const applications = ref<JobApplicationResponseDto[]>([])
const isLoading = ref<boolean>(true)
const fetchError = ref<string>('')
const reconcilingIds = ref<Record<string, boolean>>({})
const reconcileError = ref<Record<string, string>>({})

const loadApplications = async () => {
  isLoading.value = true
  fetchError.value = ''
  try {
    applications.value = await listApplications()
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { statusCode?: number; data?: { error?: { message?: string } }; message?: string }
      if (apiErr.statusCode === 401) {
        fetchError.value = t('applications.unauthorized') || 'Session expired. Please sign in.'
      } else {
        fetchError.value = apiErr.data?.error?.message || apiErr.message || t('applications.fetchError')
      }
    } else {
      fetchError.value = t('applications.fetchError')
    }
  } finally {
    isLoading.value = false
  }
}

const handleReconcile = async (appId: string, submitted: boolean) => {
  reconcilingIds.value[appId] = true
  delete reconcileError.value[appId]
  try {
    const updated = await reconcileApplication(appId, submitted)
    const idx = applications.value.findIndex((a) => a.id === appId)
    if (idx !== -1) {
      applications.value[idx] = updated
    } else {
      await loadApplications()
    }
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { data?: { error?: { message?: string } }; message?: string }
      reconcileError.value[appId] =
        apiErr.data?.error?.message || apiErr.message || t('applications.reconcileFailed')
    } else {
      reconcileError.value[appId] = t('applications.reconcileFailed')
    }
  } finally {
    reconcilingIds.value[appId] = false
  }
}

const formatDate = (dateStr: string): string => {
  try {
    return new Date(dateStr).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

const getStatusBadgeClass = (status: ApplicationStatus): string => {
  switch (status) {
    case 'submitted':
      return 'bg-emerald-50 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800'
    case 'submitting':
      return 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800 animate-pulse'
    case 'approved':
      return 'bg-blue-50 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800'
    case 'prepared':
      return 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700'
    case 'unknown':
      return 'bg-amber-50 dark:bg-amber-950/60 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800'
    case 'needs_review':
      return 'bg-rose-50 dark:bg-rose-950/60 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800'
    default:
      return 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700'
  }
}

const getSiteBadgeClass = (site: string): string => {
  switch (site.toLowerCase()) {
    case 'greenhouse':
      return 'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-800 dark:text-emerald-200'
    case 'linkedin':
      return 'bg-sky-100 dark:bg-sky-900/40 text-sky-800 dark:text-sky-200'
    case 'topcv':
      return 'bg-orange-100 dark:bg-orange-900/40 text-orange-800 dark:text-orange-200'
    default:
      return 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300'
  }
}


onMounted(() => {
  loadApplications()
})
</script>

<template>
  <div class="max-w-5xl mx-auto py-8 px-4 sm:px-6 space-y-8">
    <!-- Header -->
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 mb-2">
        <span class="px-2.5 py-0.5 text-xs font-mono font-semibold bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded">
          {{ $t('applications.badge') }}
        </span>
      </div>
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
            {{ $t('applications.title') }}
          </h1>
          <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl">
            {{ $t('applications.description') }}
          </p>
        </div>
        <button
          type="button"
          @click="loadApplications"
          :disabled="isLoading"
          class="p-1.5 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition"
          :aria-label="$t('common.refresh')"
        >
          <RefreshCw class="w-4 h-4" :class="isLoading ? 'animate-spin' : ''" />
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div
      v-if="isLoading"
      class="p-12 border border-slate-200 dark:border-slate-800 rounded-lg flex flex-col items-center justify-center text-slate-500 gap-2 bg-white dark:bg-slate-900"
    >
      <div class="animate-spin rounded-full h-5 w-5 border-2 border-indigo-600 border-t-transparent" />
      <span class="text-xs">{{ $t('applications.loading') }}</span>
    </div>

    <!-- Load Error State -->
    <div
      v-else-if="fetchError"
      role="alert"
      class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between"
    >
      <span>{{ fetchError }}</span>
      <button
        type="button"
        @click="loadApplications"
        class="ml-3 px-2.5 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium transition"
      >
        {{ $t('common.retry') }}
      </button>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="applications.length === 0"
      class="p-12 border-2 border-dashed border-slate-300 dark:border-slate-700 rounded-lg text-center space-y-2"
    >
      <Briefcase class="w-10 h-10 mx-auto text-slate-400" />
      <p class="text-xs font-medium text-slate-700 dark:text-slate-300">
        {{ $t('applications.noApplications') }}
      </p>
      <p class="text-[11px] text-slate-400 max-w-sm mx-auto">
        {{ $t('applications.noApplicationsHint') }}
      </p>
    </div>

    <!-- Applications List -->
    <div v-else class="space-y-4">
      <div
        v-for="app in applications"
        :key="app.id"
        class="border border-slate-200 dark:border-slate-800 rounded-lg p-5 bg-white dark:bg-slate-900 shadow-xs space-y-4"
      >
        <!-- Top Row: Role, Company, Site Badge, Status Badge -->
        <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-3">
          <div class="space-y-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span
                class="px-2 py-0.5 text-[10px] font-mono font-semibold uppercase rounded"
                :class="getSiteBadgeClass(app.site)"
              >
                {{ app.site }}
              </span>
              <span
                class="px-2 py-0.5 text-[10px] font-mono font-semibold uppercase rounded border"
                :class="getStatusBadgeClass(app.status)"
              >
                {{ $t(`applications.status.${app.status}`) }}
              </span>
            </div>
            <h2 class="text-sm font-bold text-slate-900 dark:text-white truncate">
              {{ app.role_title }}
            </h2>
            <div class="text-xs font-medium text-slate-600 dark:text-slate-400">
              {{ app.company_name }}
            </div>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            <a
              v-if="app.source_url && safeExternalHttpUrl(app.source_url)"
              :href="safeExternalHttpUrl(app.source_url)"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-1 text-[11px] text-slate-500 hover:text-indigo-600 dark:hover:text-indigo-400 transition"
              :aria-label="`${$t('applications.viewPosting')} on ${app.site}`"
            >
              <span>{{ $t('applications.viewPosting') }}</span>
              <ExternalLink class="w-3.5 h-3.5" />
            </a>
          </div>
        </div>

        <!-- Details & Metadata -->
        <div class="flex flex-wrap items-center gap-4 text-[11px] font-mono text-slate-500 dark:text-slate-400 pt-2 border-t border-slate-100 dark:border-slate-800">
          <div v-if="app.resume_filename" class="flex items-center gap-1 truncate max-w-xs">
            <span class="text-slate-400">{{ $t('applications.resume') }}:</span>
            <span class="text-slate-700 dark:text-slate-300 truncate">{{ app.resume_filename }}</span>
          </div>
          <div>
            <span class="text-slate-400">{{ $t('applications.recorded') }}:</span>
            <span>{{ formatDate(app.created_at) }}</span>
          </div>
        </div>

        <!-- Needs Review Banner -->
        <div
          v-if="app.status === 'needs_review'"
          role="alert"
          class="p-3.5 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-xs space-y-1"
        >
          <div class="flex items-center gap-2 font-semibold text-rose-800 dark:text-rose-200">
            <AlertTriangle class="w-4 h-4 shrink-0" />
            <span>{{ $t('applications.needsReviewTitle') }}</span>
          </div>
          <p class="text-rose-700 dark:text-rose-300 text-[11px]">
            {{ $t('applications.needsReviewDesc') }}
          </p>
        </div>

        <!-- Reconciliation Action Card for 'unknown' status -->
        <div
          v-if="app.status === 'unknown'"
          class="p-4 rounded-lg bg-amber-50/70 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 space-y-3"
        >
          <div class="flex items-start gap-2.5">
            <HelpCircle class="w-4 h-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
            <div class="space-y-1">
              <h3 class="text-xs font-bold text-amber-900 dark:text-amber-200">
                {{ $t('applications.reconcileTitle') }}
              </h3>
              <p class="text-[11px] text-amber-800 dark:text-amber-300">
                {{ $t('applications.reconcileDesc') }}
              </p>
            </div>
          </div>

          <div
            v-if="reconcileError[app.id]"
            role="alert"
            class="text-[11px] text-rose-600 dark:text-rose-400 font-medium"
          >
            {{ reconcileError[app.id] }}
          </div>

          <div class="flex items-center gap-2 pt-1">
            <AppButton
              variant="primary"
              size="xs"
              @click="handleReconcile(app.id, true)"
              :loading="reconcilingIds[app.id]"
              :disabled="reconcilingIds[app.id]"
            >
              <CheckCircle2 class="w-3.5 h-3.5 mr-1" />
              <span>{{ $t('applications.reconcileConfirmed') }}</span>
            </AppButton>
            <AppButton
              variant="outline"
              size="xs"
              @click="handleReconcile(app.id, false)"
              :loading="reconcilingIds[app.id]"
              :disabled="reconcilingIds[app.id]"
            >
              <XCircle class="w-3.5 h-3.5 mr-1" />
              <span>{{ $t('applications.reconcileNotSubmitted') }}</span>
            </AppButton>
          </div>
        </div>

        <!-- Linked Workspace: ONLY visible when status is 'submitted' AND workspace_id exists -->
        <div
          v-if="app.status === 'submitted' && app.workspace_id"
          class="p-3.5 rounded-lg bg-emerald-50/70 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800 flex items-center justify-between text-xs"
        >
          <div class="flex items-center gap-2 text-emerald-800 dark:text-emerald-200">
            <Sparkles class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
            <span class="font-medium">
              {{ $t('applications.workspaceReady') }}
            </span>
          </div>
          <NuxtLink
            :to="`/interviews/${encodeURIComponent(cleanWorkspaceId(app.workspace_id))}/overview`"
            class="inline-flex items-center gap-1 font-semibold text-emerald-700 dark:text-emerald-300 hover:text-emerald-800 dark:hover:text-emerald-200 transition"
          >
            <span>{{ $t('applications.startPrep') }}</span>
            <ArrowRight class="w-3.5 h-3.5" />
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>