<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Activity, RefreshCw } from 'lucide-vue-next'
import type { TokenUsageCursor, TokenUsageResponse, TokenUsageRow } from '@mensetsu/contracts'
import { useApiClient } from '~/services/api.client'
import { formatDisplayDateTime } from '~/utils/date'

const props = defineProps<{ scope: 'user' | 'admin' }>()
const endpoint = computed(() => props.scope === 'admin' ? '/api/v1/admin/usage/tokens' : '/api/v1/usage/tokens')
const request = useApiClient().request
const { data, pending, error, refresh } = await useAsyncData<TokenUsageResponse>(
  `token-usage-${props.scope}`,
  () => request<TokenUsageResponse>(endpoint.value),
)
const nextCursor = ref<TokenUsageCursor | null>(data.value?.next_cursor ?? null)
const olderRows = ref<TokenUsageRow[]>([])
const loadingOlder = ref(false)
const olderError = ref(false)
watch(data, (value) => { nextCursor.value = value?.next_cursor ?? null })

const loadOlder = async () => {
  if (!nextCursor.value || loadingOlder.value || pending.value) return
  loadingOlder.value = true
  olderError.value = false
  try {
    const page = await request<TokenUsageResponse>(endpoint.value, { query: nextCursor.value })
    olderRows.value.push(...page.items)
    nextCursor.value = page.next_cursor ?? null
  } catch {
    olderError.value = true
  } finally {
    loadingOlder.value = false
  }
}

const refreshPanel = async () => {
  if (pending.value || loadingOlder.value) return
  olderRows.value = []
  olderError.value = false
  await refresh()
  nextCursor.value = data.value?.next_cursor ?? null
}

const numberFormat = new Intl.NumberFormat()
const format = (value: number) => numberFormat.format(value)
const rows = computed(() => [...(data.value?.items ?? []), ...olderRows.value])
const totals = computed(() => data.value?.totals)
</script>

<template>
  <section class="overflow-hidden rounded-md border border-slate-200 bg-white text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-100" aria-labelledby="token-usage-title">
    <header class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200 px-4 py-3 dark:border-slate-800 sm:px-5">
      <div>
        <h2 id="token-usage-title" class="text-sm font-semibold text-slate-900 dark:text-white">{{ scope === 'admin' ? 'Platform token usage' : 'Token usage' }}</h2>
        <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">Recorded model requests and token counts</p>
      </div>
      <button type="button" class="inline-flex min-h-9 items-center gap-2 rounded border border-slate-300 px-3 text-xs font-medium text-slate-700 hover:bg-slate-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:opacity-60 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800 dark:focus-visible:outline-indigo-400" :disabled="pending || loadingOlder" @click="refreshPanel()">
        <RefreshCw class="h-3.5 w-3.5" :class="pending ? 'animate-spin' : ''" aria-hidden="true" />
        Refresh
      </button>
    </header>

    <div v-if="pending && !data" class="grid grid-cols-2 divide-x divide-y divide-slate-200 dark:divide-slate-800 sm:grid-cols-4 sm:divide-y-0" aria-label="Loading token usage">
      <div v-for="index in 4" :key="index" class="min-h-[76px] p-3 sm:p-4">
        <div class="h-3 w-20 rounded bg-slate-200 dark:bg-slate-800" /><div class="mt-3 h-5 w-16 rounded bg-slate-200 dark:bg-slate-800" />
      </div>
    </div>
    <div v-else-if="error" class="m-4 rounded border border-rose-200 bg-rose-50 px-3 py-2.5 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-200" role="alert">
      Token usage could not be loaded. Try refreshing the panel.
    </div>
    <dl v-else class="grid grid-cols-2 divide-x divide-y divide-slate-200 dark:divide-slate-800 sm:grid-cols-4 sm:divide-y-0">
      <div class="min-h-[64px] px-3 py-2.5 sm:px-4"><dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">Requests</dt><dd class="mt-1 font-mono text-base font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.request_count ?? 0) }}</dd></div>
      <div class="min-h-[64px] px-3 py-2.5 sm:px-4"><dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">Input tokens</dt><dd class="mt-1 font-mono text-base font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.input_tokens ?? 0) }}</dd></div>
      <div class="min-h-[64px] px-3 py-2.5 sm:px-4"><dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">Output tokens</dt><dd class="mt-1 font-mono text-base font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.output_tokens ?? 0) }}</dd></div>
      <div class="min-h-[64px] px-3 py-2.5 sm:px-4"><dt class="text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">All tokens</dt><dd class="mt-1 font-mono text-base font-semibold tabular-nums text-emerald-600 dark:text-emerald-300">{{ format(totals?.total_tokens ?? 0) }}</dd></div>
    </dl>

    <div v-if="!pending && !error && rows.length === 0" class="border-t border-slate-200 px-4 py-5 text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
      No model request usage has been recorded yet.
    </div>
    <div v-else-if="rows.length" class="max-h-[28rem] overflow-x-auto overflow-y-auto overscroll-contain focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-indigo-500" role="region" aria-label="Token usage records" tabindex="0">
      <table class="w-full min-w-[760px] border-collapse text-left text-[11px]">
        <thead class="sticky top-0 z-10 bg-slate-50 text-[10px] uppercase tracking-wide text-slate-500 dark:bg-slate-800 dark:text-slate-400">
          <tr><th scope="col" class="px-3 py-2 font-medium">Time</th><th v-if="scope === 'admin'" scope="col" class="px-3 py-2 font-medium">User ID</th><th scope="col" class="px-3 py-2 font-medium">Model</th><th scope="col" class="px-3 py-2 font-medium">Agent</th><th scope="col" class="px-3 py-2 text-right font-medium">Input</th><th scope="col" class="px-3 py-2 text-right font-medium">Output</th><th scope="col" class="px-3 py-2 text-right font-medium">Total</th></tr>
        </thead>
        <tbody class="divide-y divide-slate-200 dark:divide-slate-800">
          <tr v-for="item in rows" :key="item.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50">
            <td class="whitespace-nowrap px-3 py-2.5 font-mono tabular-nums text-slate-500 dark:text-slate-400">{{ formatDisplayDateTime(item.timestamp) }}</td>
            <td v-if="scope === 'admin'" class="max-w-48 truncate px-3 py-2.5 font-mono text-slate-600 dark:text-slate-300" :title="item.user_id || 'Unknown user'">{{ item.user_id || 'Unknown user' }}</td>
            <td class="px-3 py-2.5"><div class="font-medium text-slate-900 dark:text-slate-100">{{ item.model_name }}</div><div class="text-[10px] text-slate-500 dark:text-slate-400">{{ item.model_provider }}</div></td>
            <td class="px-3 py-2.5 font-mono text-slate-600 dark:text-slate-300">{{ item.agent_id }}</td>
            <td class="px-3 py-2.5 text-right font-mono tabular-nums text-slate-600 dark:text-slate-300">{{ format(item.input_tokens) }}</td>
            <td class="px-3 py-2.5 text-right font-mono tabular-nums text-slate-600 dark:text-slate-300">{{ format(item.output_tokens) }}</td>
            <td class="px-3 py-2.5 text-right font-mono font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(item.total_tokens) }}</td>
          </tr>
        </tbody>
      </table>
      <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-slate-200 px-3 py-2.5 dark:border-slate-800">
        <p v-if="olderError" class="text-xs text-rose-600 dark:text-rose-300" role="alert">Older requests could not be loaded.</p>
        <p v-else class="text-[11px] text-slate-500 dark:text-slate-400">{{ rows.length }} requests loaded; aggregate covers {{ format(totals?.request_count ?? 0) }}.</p>
        <button v-if="nextCursor" type="button" class="min-h-9 rounded border border-slate-300 px-3 text-xs font-medium text-slate-700 hover:bg-slate-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:ring-indigo-500 disabled:opacity-60 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800 dark:focus-visible:outline-indigo-400" :disabled="loadingOlder || pending" @click="loadOlder()">
          {{ loadingOlder ? 'Loading…' : 'Load older requests' }}
        </button>
      </footer>
    </div>
  </section>
</template>
