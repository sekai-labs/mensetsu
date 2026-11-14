<script setup lang="ts">
import { ref, computed } from 'vue'
import { RefreshCw, ArrowUpRight, Activity, ShieldCheck, Server, Cpu } from 'lucide-vue-next'
import { useAuthService } from '~/features/auth/services/auth.service'
import { useUsageService } from '~/features/usage/services/usage.service'
import type { MetricsSummary } from '~/models/metrics.model'

const { user } = useAuthService()
const { getMetricsSummary } = useUsageService()

const isAdmin = computed(() => user.value?.role === 'admin')

const summary = ref<MetricsSummary | null>(null)
const pending = ref(true)
const error = ref<string | null>(null)

const numberFormat = new Intl.NumberFormat()
const formatNumber = (num?: number) => numberFormat.format(num ?? 0)

const fetchSummary = async () => {
  pending.value = true
  error.value = null
  try {
    summary.value = await getMetricsSummary(isAdmin.value)
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      error.value = err.message
    } else {
      error.value = 'Failed to load metrics summary from server.'
    }
  } finally {
    pending.value = false
  }
}

// Initial fetch on mount
onMounted(() => {
  fetchSummary()
})
</script>

<template>
  <section
    class="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900"
    aria-labelledby="metrics-summary-title"
  >
    <header class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-100 px-4 py-3 dark:border-slate-800/80 sm:px-5">
      <div class="flex items-center gap-2.5">
        <div class="flex h-7 w-7 items-center justify-center rounded bg-indigo-50 text-indigo-600 dark:bg-indigo-950/60 dark:text-indigo-400">
          <Activity class="h-4 w-4" aria-hidden="true" />
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h2 id="metrics-summary-title" class="text-xs font-semibold uppercase tracking-[0.14em] text-slate-800 dark:text-slate-200">
              {{ isAdmin ? $t('dashboard.metrics.platformTitle') : $t('dashboard.metrics.accountTitle') }}
            </h2>
            <span
              class="inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-mono font-medium"
              :class="isAdmin
                ? 'bg-purple-50 text-purple-700 dark:bg-purple-950/60 dark:text-purple-300 border border-purple-200 dark:border-purple-800'
                : 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300 border border-slate-200 dark:border-slate-700'"
            >
              {{ isAdmin ? $t('dashboard.metrics.scopePlatform') : $t('dashboard.metrics.scopeUser') }}
            </span>
          </div>
          <p class="text-[11px] text-slate-500 dark:text-slate-400">
            {{ isAdmin ? $t('dashboard.metrics.platformSubtitle') : $t('dashboard.metrics.accountSubtitle') }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="inline-flex h-8 items-center gap-1.5 rounded border border-slate-200 px-2.5 text-xs font-medium text-slate-600 hover:bg-slate-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 disabled:opacity-50 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
          :disabled="pending"
          :aria-label="$t('common.refresh')"
          @click="fetchSummary()"
        >
          <RefreshCw class="h-3.5 w-3.5" :class="pending ? 'animate-spin text-indigo-600' : ''" aria-hidden="true" />
          <span class="hidden sm:inline">{{ $t('common.refresh') }}</span>
        </button>

        <NuxtLink
          to="/settings?tab=usage"
          class="inline-flex h-8 items-center gap-1 rounded bg-indigo-50 px-2.5 text-xs font-medium text-indigo-700 hover:bg-indigo-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 dark:bg-indigo-950/60 dark:text-indigo-300 dark:hover:bg-indigo-900/60 transition-colors"
        >
          <span>{{ $t('dashboard.metrics.viewDetail') }}</span>
          <ArrowUpRight class="h-3.5 w-3.5" aria-hidden="true" />
        </NuxtLink>
      </div>
    </header>

    <!-- Loading State -->
    <div
      v-if="pending && !summary"
      class="grid grid-cols-2 divide-x divide-y divide-slate-100 dark:divide-slate-800 sm:grid-cols-4 sm:divide-y-0"
      aria-label="Loading metrics"
    >
      <div v-for="i in 4" :key="i" class="p-4 space-y-2 animate-pulse">
        <div class="h-3 w-16 rounded bg-slate-200 dark:bg-slate-800" />
        <div class="h-6 w-24 rounded bg-slate-200 dark:bg-slate-800" />
      </div>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error && !summary"
      class="m-4 flex items-center justify-between gap-3 rounded-md border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-200"
      role="alert"
    >
      <span>{{ error }}</span>
      <button
        type="button"
        class="font-medium underline hover:text-rose-800 dark:hover:text-rose-100"
        @click="fetchSummary()"
      >
        {{ $t('common.retry') }}
      </button>
    </div>

    <!-- Content -->
    <div v-else-if="summary" class="divide-y divide-slate-100 dark:divide-slate-800">
      <!-- Token Usage Metrics -->
      <dl class="grid grid-cols-2 divide-x divide-y divide-slate-100 dark:divide-slate-800 sm:grid-cols-4 sm:divide-y-0">
        <div class="px-4 py-3 sm:px-5">
          <dt class="text-[10px] font-semibold uppercase tracking-[0.14em] text-slate-500 dark:text-slate-400">
            {{ $t('dashboard.metrics.totalTokens') }}
          </dt>
          <dd class="mt-1 font-mono text-xl font-bold tabular-nums text-indigo-600 dark:text-indigo-400">
            {{ formatNumber(summary.token_usage.total_tokens) }}
          </dd>
        </div>

        <div class="px-4 py-3 sm:px-5">
          <dt class="text-[10px] font-semibold uppercase tracking-[0.14em] text-slate-500 dark:text-slate-400">
            {{ $t('dashboard.metrics.inputTokens') }}
          </dt>
          <dd class="mt-1 font-mono text-xl font-semibold tabular-nums text-slate-900 dark:text-white">
            {{ formatNumber(summary.token_usage.input_tokens) }}
          </dd>
        </div>

        <div class="px-4 py-3 sm:px-5">
          <dt class="text-[10px] font-semibold uppercase tracking-[0.14em] text-slate-500 dark:text-slate-400">
            {{ $t('dashboard.metrics.outputTokens') }}
          </dt>
          <dd class="mt-1 font-mono text-xl font-semibold tabular-nums text-slate-900 dark:text-white">
            {{ formatNumber(summary.token_usage.output_tokens) }}
          </dd>
        </div>

        <div class="px-4 py-3 sm:px-5">
          <dt class="text-[10px] font-semibold uppercase tracking-[0.14em] text-slate-500 dark:text-slate-400">
            {{ $t('dashboard.metrics.requestCount') }}
          </dt>
          <dd class="mt-1 font-mono text-xl font-semibold tabular-nums text-slate-900 dark:text-white">
            {{ formatNumber(summary.token_usage.request_count) }}
          </dd>
        </div>
      </dl>

      <!-- Admin Platform HTTP Service Metrics (Only rendered when scope === 'platform' and http != null) -->
      <div v-if="summary.scope === 'platform' && summary.http" class="bg-slate-50/60 p-4 dark:bg-slate-900/50 sm:px-5">
        <div class="flex items-center gap-2 mb-2.5">
          <Server class="h-3.5 w-3.5 text-purple-600 dark:text-purple-400" aria-hidden="true" />
          <span class="text-[11px] font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
            {{ $t('dashboard.metrics.httpPlatformHealth') }}
          </span>
        </div>

        <dl class="grid grid-cols-2 gap-3 sm:grid-cols-2">
          <div class="rounded border border-slate-200/80 bg-white p-2.5 dark:border-slate-800 dark:bg-slate-850">
            <dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">
              {{ $t('dashboard.metrics.successRate') }}
            </dt>
            <dd class="mt-1 font-mono text-base font-semibold tabular-nums text-emerald-600 dark:text-emerald-400">
              {{ summary.http.success_rate_percent.toFixed(1) }}%
            </dd>
          </div>

          <div class="rounded border border-slate-200/80 bg-white p-2.5 dark:border-slate-800 dark:bg-slate-850">
            <dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">
              {{ $t('dashboard.metrics.rps') }}
            </dt>
            <dd class="mt-1 font-mono text-base font-semibold tabular-nums text-slate-900 dark:text-white">
              {{ summary.http.requests_per_second.toFixed(2) }}
            </dd>
          </div>
        </dl>
      </div>

      <footer class="flex flex-wrap items-center justify-end gap-3 px-4 py-2.5 text-[11px] text-slate-500 dark:text-slate-400 sm:px-5">
        <NuxtLink
          to="/settings?tab=usage"
          class="font-medium text-indigo-600 hover:text-indigo-700 dark:text-indigo-400 dark:hover:text-indigo-300 flex items-center gap-1"
        >
          <span>{{ isAdmin ? $t('dashboard.metrics.explorePlatformTokens') : $t('dashboard.metrics.exploreUserTokens') }}</span>
          <ArrowUpRight class="h-3 w-3" aria-hidden="true" />
        </NuxtLink>
      </footer>
    </div>
  </section>
</template>
