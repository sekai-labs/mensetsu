<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  Coins,
  Search,
  Filter,
  X,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  Shield,
} from 'lucide-vue-next'
import type { TokenUsageRow, TokenUsageTotals, TokenUsageCursor } from '@mensetsu/contracts'
import { useAuthService } from '~/features/auth/services/auth.service'
import { useUsageService } from '~/features/usage/services/usage.service'
import type { DailyTokenUsageItem, TokenUsageFilterQuery } from '~/models/metrics.model'
import TokenUsageDailyChart from '~/components/usage/TokenUsageDailyChart.vue'
import { formatDisplayDateTime } from '~/utils/date'

const { user } = useAuthService()
const { getDailyTokenUsage, getTokenUsagePage } = useUsageService()

const isAdmin = computed(() => user.value?.role === 'admin')

// Analytics 30-day Chart State
const analyticsItems = ref<DailyTokenUsageItem[]>([])
const analyticsTotals = ref<TokenUsageTotals | null>(null)
const analyticsLoading = ref(true)
const analyticsError = ref<string | null>(null)

const loadAnalytics = async () => {
  analyticsLoading.value = true
  analyticsError.value = null
  try {
    const data = await getDailyTokenUsage(isAdmin.value)
    analyticsItems.value = data.items || []
    analyticsTotals.value = data.totals || null
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      analyticsError.value = err.message
    } else {
      analyticsError.value = 'Failed to load daily token usage analytics.'
    }
  } finally {
    analyticsLoading.value = false
  }
}

// Filters & Server-Side Search State
const PAGE_SIZE = 25

const search = ref('')
const provider = ref('')
const agent = ref('')
const fromDate = ref('')
const toDate = ref('')
const userId = ref('')

// Active filter state used for queries
const activeFilters = ref<TokenUsageFilterQuery>({})

// 25-Row Cursor Pagination with Previous / Next
const pageNumber = ref(1)
const cursorStack = ref<Array<{ before_timestamp: string; before_event_id: string } | null>>([null])
const nextCursor = ref<TokenUsageCursor | null>(null)

// Table State
const items = ref<TokenUsageRow[]>([])
const totals = ref<TokenUsageTotals | null>(null)
const tableLoading = ref(true)
const tableError = ref<string | null>(null)

const numberFormat = new Intl.NumberFormat()
const formatNumber = (num?: number) => numberFormat.format(num ?? 0)

const hasActiveFilters = computed(() => {
  return Boolean(
    search.value ||
    provider.value ||
    agent.value ||
    fromDate.value ||
    toDate.value ||
    (isAdmin.value && userId.value)
  )
})

const loadTablePage = async () => {
  tableLoading.value = true
  tableError.value = null

  const currentCursor = cursorStack.value[pageNumber.value - 1] ?? null
  const query: TokenUsageFilterQuery = {
    ...activeFilters.value,
    limit: PAGE_SIZE,
    before_timestamp: currentCursor ? currentCursor.before_timestamp : undefined,
    before_event_id: currentCursor ? currentCursor.before_event_id : undefined,
  }

  try {
    const res = await getTokenUsagePage(query, isAdmin.value)
    items.value = res.items || []
    totals.value = res.totals || null
    nextCursor.value = res.next_cursor ?? null

    if (nextCursor.value && cursorStack.value.length === pageNumber.value) {
      cursorStack.value.push({
        before_timestamp: nextCursor.value.before_timestamp,
        before_event_id: nextCursor.value.before_event_id,
      })
    }
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      tableError.value = err.message
    } else {
      tableError.value = 'Failed to load token usage records.'
    }
  } finally {
    tableLoading.value = false
  }
}

const applyFilters = () => {
  activeFilters.value = {
    search: search.value.trim() || undefined,
    provider: provider.value.trim() || undefined,
    agent: agent.value.trim() || undefined,
    from_date: fromDate.value || undefined,
    to_date: toDate.value || undefined,
    user_id: isAdmin.value && userId.value.trim() ? userId.value.trim() : undefined,
  }
  pageNumber.value = 1
  cursorStack.value = [null]
  nextCursor.value = null
  loadTablePage()
}

const clearFilters = () => {
  search.value = ''
  provider.value = ''
  agent.value = ''
  fromDate.value = ''
  toDate.value = ''
  userId.value = ''
  applyFilters()
}

const handlePrevious = () => {
  if (pageNumber.value > 1 && !tableLoading.value) {
    pageNumber.value--
    loadTablePage()
  }
}

const handleNext = () => {
  if (nextCursor.value && !tableLoading.value) {
    pageNumber.value++
    loadTablePage()
  }
}

const refreshAll = async () => {
  await Promise.all([loadAnalytics(), loadTablePage()])
}

onMounted(() => {
  loadAnalytics()
  loadTablePage()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header Block -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-surface-border pb-4">
      <div>
        <div class="flex items-center gap-2">
          <h2 class="text-base font-bold text-text-primary">
            {{ isAdmin ? $t('usage.titleAdmin') : $t('usage.titleUser') }}
          </h2>
          <span
            class="inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-mono font-medium"
            :class="isAdmin
              ? 'bg-purple-100 text-purple-800 dark:bg-purple-950/80 dark:text-purple-300 border border-purple-200 dark:border-purple-800'
              : 'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/60 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800'"
          >
            <Shield v-if="isAdmin" class="h-3 w-3" aria-hidden="true" />
            {{ isAdmin ? $t('usage.scopePlatform') : $t('usage.scopeUser') }}
          </span>
        </div>
        <p class="text-xs text-text-muted mt-1">
          {{ isAdmin ? $t('usage.subtitleAdmin') : $t('usage.subtitleUser') }}
        </p>
      </div>

      <button
        type="button"
        class="inline-flex h-8 items-center gap-1.5 self-start sm:self-auto rounded-md border border-surface-border bg-surface-base px-2.5 text-xs font-medium text-text-primary shadow-sm hover:bg-surface-elevated focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-50"
        :disabled="tableLoading || analyticsLoading"
        @click="refreshAll()"
      >
        <RefreshCw
          class="h-3.5 w-3.5"
          :class="tableLoading || analyticsLoading ? 'animate-spin text-primary' : ''"
          aria-hidden="true"
        />
        <span>{{ $t('common.refresh') }}</span>
      </button>
    </div>

    <!-- Scope Totals Overview Cards -->
    <section aria-labelledby="usage-totals-heading" class="grid grid-cols-2 gap-3 sm:grid-cols-4">
      <h3 id="usage-totals-heading" class="sr-only">Aggregate Token Totals</h3>

      <div class="rounded-lg border border-surface-border border-l-2 border-l-primary bg-surface-elevated/70 p-4 shadow-sm">
        <div class="text-[10px] font-semibold uppercase tracking-[0.14em] text-text-muted">
          {{ $t('usage.totals.allTokens') }}
        </div>
        <div class="mt-2 font-mono text-xl font-bold tracking-tight text-primary tabular-nums">
          {{ formatNumber(totals?.total_tokens) }}
        </div>
      </div>

      <div class="rounded-lg border border-surface-border border-l-2 border-l-slate-400 bg-surface-elevated/70 p-4 shadow-sm">
        <div class="text-[10px] font-semibold uppercase tracking-[0.14em] text-text-muted">
          {{ $t('usage.totals.inputTokens') }}
        </div>
        <div class="mt-2 font-mono text-xl font-semibold tracking-tight text-text-primary tabular-nums">
          {{ formatNumber(totals?.input_tokens) }}
        </div>
      </div>

      <div class="rounded-lg border border-surface-border border-l-2 border-l-slate-400 bg-surface-elevated/70 p-4 shadow-sm">
        <div class="text-[10px] font-semibold uppercase tracking-[0.14em] text-text-muted">
          {{ $t('usage.totals.outputTokens') }}
        </div>
        <div class="mt-2 font-mono text-xl font-semibold tracking-tight text-text-primary tabular-nums">
          {{ formatNumber(totals?.output_tokens) }}
        </div>
      </div>

      <div class="rounded-lg border border-surface-border border-l-2 border-l-emerald-500 bg-surface-elevated/70 p-4 shadow-sm">
        <div class="text-[10px] font-semibold uppercase tracking-[0.14em] text-text-muted">
          {{ $t('usage.totals.requests') }}
        </div>
        <div class="mt-2 font-mono text-xl font-semibold tracking-tight text-emerald-600 dark:text-emerald-400 tabular-nums">
          {{ formatNumber(totals?.request_count) }}
        </div>
      </div>
    </section>

    <!-- 30-Day Daily Token Usage Chart -->
    <TokenUsageDailyChart
      :items="analyticsItems"
      :totals="analyticsTotals"
      :loading="analyticsLoading"
      :error="analyticsError"
      @retry="loadAnalytics"
    />

    <!-- Filter & Server-side Search Controls -->
    <section
      class="rounded-lg border border-surface-border bg-surface-elevated/70 p-4 shadow-sm"
      aria-label="Filter token records"
    >
      <form class="space-y-3" @submit.prevent="applyFilters">
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
          <!-- Server Search -->
          <div class="sm:col-span-2">
            <label for="search-input" class="block text-[11px] font-medium uppercase tracking-wider text-text-muted mb-1">
              {{ $t('common.search') }}
            </label>
            <div class="relative">
              <input
                id="search-input"
                v-model="search"
                type="search"
                :placeholder="$t('usage.filters.searchPlaceholder')"
                class="w-full rounded-md border border-surface-border bg-surface-base py-1.5 pl-8 pr-3 text-xs text-text-primary placeholder-text-muted focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <Search class="pointer-events-none absolute left-2.5 top-2 h-3.5 w-3.5 text-text-muted" aria-hidden="true" />
            </div>
          </div>

          <!-- Provider Filter -->
          <div>
            <label for="provider-filter" class="block text-[11px] font-medium uppercase tracking-wider text-text-muted mb-1">
              {{ $t('usage.filters.provider') }}
            </label>
            <input
              id="provider-filter"
              v-model="provider"
              type="text"
              placeholder="e.g. google, openai"
              class="w-full rounded-md border border-surface-border bg-surface-base px-2.5 py-1.5 text-xs text-text-primary placeholder-text-muted focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Agent Filter -->
          <div>
            <label for="agent-filter" class="block text-[11px] font-medium uppercase tracking-wider text-text-muted mb-1">
              {{ $t('usage.filters.agent') }}
            </label>
            <input
              id="agent-filter"
              v-model="agent"
              type="text"
              placeholder="e.g. planner, curriculum"
              class="w-full rounded-md border border-surface-border bg-surface-base px-2.5 py-1.5 text-xs text-text-primary placeholder-text-muted focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Date From -->
          <div>
            <label for="from-date-filter" class="block text-[11px] font-medium uppercase tracking-wider text-text-muted mb-1">
              {{ $t('usage.filters.fromDate') }}
            </label>
            <input
              id="from-date-filter"
              v-model="fromDate"
              type="date"
              class="w-full rounded-md border border-surface-border bg-surface-base px-2.5 py-1.5 text-xs text-text-primary focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Date To -->
          <div>
            <label for="to-date-filter" class="block text-[11px] font-medium uppercase tracking-wider text-text-muted mb-1">
              {{ $t('usage.filters.toDate') }}
            </label>
            <input
              id="to-date-filter"
              v-model="toDate"
              type="date"
              class="w-full rounded-md border border-surface-border bg-surface-base px-2.5 py-1.5 text-xs text-text-primary focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- User ID Filter (ADMIN ONLY) -->
          <div v-if="isAdmin" class="sm:col-span-2 md:col-span-1">
            <label for="user-id-filter" class="block text-[11px] font-medium uppercase tracking-wider text-purple-700 dark:text-purple-400 mb-1">
              {{ $t('usage.filters.userId') }}
            </label>
            <input
              id="user-id-filter"
              v-model="userId"
              type="text"
              placeholder="e.g. user:..."
              class="w-full rounded-md border border-purple-200 bg-purple-50/40 px-2.5 py-1.5 text-xs text-text-primary placeholder-text-muted focus:border-purple-500 focus:outline-none focus:ring-1 focus:ring-purple-500 dark:border-purple-900 dark:bg-purple-950/20"
            />
          </div>
        </div>

        <div class="flex items-center justify-between pt-1 border-t border-surface-border">
          <div class="text-[11px] text-text-muted">
            <span v-if="hasActiveFilters" class="font-medium text-primary">
              {{ $t('usage.filters.activeIndicator') }}
            </span>
          </div>

          <div class="flex items-center gap-2">
            <button
              v-if="hasActiveFilters"
              type="button"
              class="inline-flex items-center gap-1 rounded px-2.5 py-1 text-xs text-text-muted hover:bg-surface-elevated hover:text-text-primary"
              @click="clearFilters"
            >
              <X class="h-3 w-3" aria-hidden="true" />
              <span>{{ $t('common.clear') }}</span>
            </button>

            <button
              type="submit"
              class="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-60"
              :disabled="tableLoading"
            >
              <Filter class="h-3.5 w-3.5" aria-hidden="true" />
              <span>{{ $t('usage.filters.apply') }}</span>
            </button>
          </div>
        </div>
      </form>
    </section>

    <!-- Table Section with 25-row Previous/Next Cursor Pagination -->
    <section
      class="overflow-hidden rounded-lg border border-surface-border bg-surface-elevated/70 shadow-sm"
      aria-labelledby="token-records-title"
    >
      <header class="flex flex-wrap items-center justify-between gap-3 border-b border-surface-border px-4 py-3 sm:px-5">
        <div>
          <h3 id="token-records-title" class="text-xs font-semibold uppercase tracking-[0.14em] text-text-primary">
            {{ $t('usage.table.title') }}
          </h3>
          <p class="text-[11px] text-text-muted">
            {{ $t('usage.table.subtitle', { size: PAGE_SIZE }) }}
          </p>
        </div>

        <!-- Pagination Controls -->
        <nav aria-label="Pagination Navigation" class="flex items-center gap-2">
          <span class="text-xs text-text-muted font-mono">
            {{ $t('usage.pagination.pageLabel', { page: pageNumber }) }}
          </span>

          <div class="inline-flex rounded-md shadow-sm">
            <button
              type="button"
              class="inline-flex items-center gap-1 rounded-l-md border border-surface-border bg-surface-base px-2.5 py-1 text-xs font-medium text-text-primary hover:bg-surface-elevated focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="pageNumber <= 1 || tableLoading"
              :aria-label="$t('common.previous')"
              @click="handlePrevious"
            >
              <ChevronLeft class="h-3.5 w-3.5" aria-hidden="true" />
              <span class="hidden sm:inline">{{ $t('common.previous') }}</span>
            </button>

            <button
              type="button"
              class="inline-flex items-center gap-1 rounded-r-md border border-l-0 border-surface-border bg-surface-base px-2.5 py-1 text-xs font-medium text-text-primary hover:bg-surface-elevated focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="!nextCursor || tableLoading"
              :aria-label="$t('common.next')"
              @click="handleNext"
            >
              <span class="hidden sm:inline">{{ $t('common.next') }}</span>
              <ChevronRight class="h-3.5 w-3.5" aria-hidden="true" />
            </button>
          </div>
        </nav>
      </header>

      <!-- Loading State -->
      <div v-if="tableLoading && items.length === 0" class="p-8 text-center" aria-live="polite">
        <div class="inline-flex items-center gap-2 text-xs text-text-muted">
          <RefreshCw class="h-4 w-4 animate-spin text-primary" aria-hidden="true" />
          <span>{{ $t('common.loading') }}</span>
        </div>
      </div>

      <!-- Error State -->
      <div
        v-else-if="tableError"
        class="m-4 flex items-center justify-between rounded-md border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-200"
        role="alert"
      >
        <span>{{ tableError }}</span>
        <button
          type="button"
          class="font-medium underline hover:text-rose-800 dark:hover:text-rose-100"
          @click="loadTablePage"
        >
          {{ $t('common.retry') }}
        </button>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="!tableLoading && items.length === 0"
        class="p-10 text-center"
      >
        <p class="text-xs text-text-muted">
          {{ hasActiveFilters ? $t('usage.table.emptyFiltered') : $t('usage.table.empty') }}
        </p>
        <button
          v-if="hasActiveFilters"
          type="button"
          class="mt-2 text-xs font-medium text-primary hover:underline"
          @click="clearFilters"
        >
          {{ $t('usage.filters.clearAction') }}
        </button>
      </div>

      <!-- Table -->
      <div
        v-else
        class="overflow-x-auto focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
        role="region"
        aria-label="Token usage records table"
        tabindex="0"
      >
        <table class="w-full min-w-[760px] border-collapse text-left text-xs">
          <thead class="sticky top-0 z-10 bg-surface-elevated text-[10px] uppercase tracking-wider text-text-muted">
            <tr>
              <th scope="col" class="px-3.5 py-2.5 font-medium">{{ $t('usage.table.time') }}</th>
              <th v-if="isAdmin" scope="col" class="px-3.5 py-2.5 font-medium">{{ $t('usage.table.userId') }}</th>
              <th scope="col" class="px-3.5 py-2.5 font-medium">{{ $t('usage.table.model') }}</th>
              <th scope="col" class="px-3.5 py-2.5 font-medium">{{ $t('usage.table.agent') }}</th>
              <th scope="col" class="px-3.5 py-2.5 text-right font-medium">{{ $t('usage.table.input') }}</th>
              <th scope="col" class="px-3.5 py-2.5 text-right font-medium">{{ $t('usage.table.output') }}</th>
              <th scope="col" class="px-3.5 py-2.5 text-right font-medium">{{ $t('usage.table.total') }}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-surface-border">
            <tr
              v-for="item in items"
              :key="item.id"
              class="hover:bg-surface-elevated transition-colors"
            >
              <td class="whitespace-nowrap px-3.5 py-2.5 font-mono text-[11px] tabular-nums text-text-muted">
                {{ formatDisplayDateTime(item.timestamp) }}
              </td>

              <td
                v-if="isAdmin"
                class="max-w-44 truncate px-3.5 py-2.5 font-mono text-[11px] text-text-muted"
                :title="item.user_id || 'Unknown'"
              >
                {{ item.user_id || '—' }}
              </td>

              <td class="px-3.5 py-2.5">
                <div class="font-medium text-text-primary">{{ item.model_name }}</div>
                <div class="text-[10px] text-text-muted font-mono">{{ item.model_provider }}</div>
              </td>

              <td class="px-3.5 py-2.5 font-mono text-[11px] text-text-muted">
                <span class="rounded bg-surface-base border border-surface-border px-1.5 py-0.5">
                  {{ item.agent_id }}
                </span>
              </td>

              <td class="px-3.5 py-2.5 text-right font-mono text-[11px] tabular-nums text-text-muted">
                {{ formatNumber(item.input_tokens) }}
              </td>

              <td class="px-3.5 py-2.5 text-right font-mono text-[11px] tabular-nums text-text-muted">
                {{ formatNumber(item.output_tokens) }}
              </td>

              <td class="px-3.5 py-2.5 text-right font-mono text-[11px] font-semibold tabular-nums text-text-primary">
                {{ formatNumber(item.total_tokens) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Table Footer with row count & pagination -->
      <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-surface-border px-4 py-3 text-xs text-text-muted sm:px-5">
        <div>
          {{ $t('usage.pagination.summary', { count: items.length, total: formatNumber(totals?.request_count) }) }}
        </div>

        <nav aria-label="Pagination Navigation Footer" class="flex items-center gap-2">
          <button
            type="button"
            class="inline-flex items-center gap-1 rounded border border-surface-border bg-surface-base px-2.5 py-1 text-xs font-medium text-text-primary hover:bg-surface-elevated focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-40 disabled:cursor-not-allowed"
            :disabled="pageNumber <= 1 || tableLoading"
            @click="handlePrevious"
          >
            <ChevronLeft class="h-3.5 w-3.5" aria-hidden="true" />
            <span>{{ $t('common.previous') }}</span>
          </button>

          <span class="font-mono text-xs px-1">
            {{ pageNumber }}
          </span>

          <button
            type="button"
            class="inline-flex items-center gap-1 rounded border border-surface-border bg-surface-base px-2.5 py-1 text-xs font-medium text-text-primary hover:bg-surface-elevated focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-40 disabled:cursor-not-allowed"
            :disabled="!nextCursor || tableLoading"
            @click="handleNext"
          >
            <span>{{ $t('common.next') }}</span>
            <ChevronRight class="h-3.5 w-3.5" aria-hidden="true" />
          </button>
        </nav>
      </footer>
    </section>
  </div>
</template>
