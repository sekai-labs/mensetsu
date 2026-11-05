<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Activity, RefreshCw } from 'lucide-vue-next'
import type { TokenUsageCursor, TokenUsageResponse, TokenUsageRow } from '@mensetsu/contracts'
import { useApiClient } from '~/services/api.client'

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
  if (!nextCursor.value || loadingOlder.value) return
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
  olderRows.value = []
  olderError.value = false
  await refresh()
  nextCursor.value = data.value?.next_cursor ?? null
}

const numberFormat = new Intl.NumberFormat()
const format = (value: number) => numberFormat.format(value)
const rows = computed(() => [...(data.value?.items ?? []), ...olderRows.value])
const totals = computed(() => data.value?.totals)
const formatDate = (value: string) => new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
}).format(new Date(value))
</script>

<template>
  <section class="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900" aria-labelledby="token-usage-title">
    <div class="flex flex-wrap items-start justify-between gap-3 border-b border-slate-200 px-4 py-3 dark:border-slate-800">
      <div class="flex items-start gap-2.5">
        <Activity class="mt-0.5 h-4 w-4 text-indigo-600 dark:text-indigo-400" aria-hidden="true" />
        <div>
          <h2 id="token-usage-title" class="text-sm font-semibold text-slate-900 dark:text-white">{{ scope === 'admin' ? 'Platform token usage' : 'Your token usage' }}</h2>
          <p class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">Per-model-request totals from recorded provider responses.</p>
        </div>
      </div>
      <button type="button" class="inline-flex items-center gap-1.5 rounded border border-slate-200 px-2 py-1 text-xs font-medium text-slate-600 hover:bg-slate-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-indigo-600 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800" :disabled="pending" @click="refreshPanel()">
        <RefreshCw class="h-3.5 w-3.5" :class="pending ? 'animate-spin' : ''" aria-hidden="true" />
        Refresh
      </button>
    </div>

    <div v-if="pending && !data" class="grid grid-cols-2 gap-px bg-slate-200 sm:grid-cols-4 dark:bg-slate-800" aria-label="Loading token usage">
      <div v-for="index in 4" :key="index" class="h-[68px] animate-pulse bg-white p-3 dark:bg-slate-900"><div class="h-3 w-20 rounded bg-slate-200 dark:bg-slate-700" /><div class="mt-3 h-4 w-14 rounded bg-slate-200 dark:bg-slate-700" /></div>
    </div>
    <div v-else-if="error" class="m-4 rounded border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950/30 dark:text-rose-300" role="alert">
      Token usage could not be loaded. Try refreshing the panel.
    </div>
    <div v-else class="grid grid-cols-2 gap-px bg-slate-200 sm:grid-cols-4 dark:bg-slate-800">
      <div class="bg-white px-3 py-2.5 dark:bg-slate-900"><div class="text-[11px] font-medium uppercase tracking-wide text-slate-500">Requests</div><div class="mt-1 font-mono text-lg font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.request_count ?? 0) }}</div></div>
      <div class="bg-white px-3 py-2.5 dark:bg-slate-900"><div class="text-[11px] font-medium uppercase tracking-wide text-slate-500">Input tokens</div><div class="mt-1 font-mono text-lg font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.input_tokens ?? 0) }}</div></div>
      <div class="bg-white px-3 py-2.5 dark:bg-slate-900"><div class="text-[11px] font-medium uppercase tracking-wide text-slate-500">Output tokens</div><div class="mt-1 font-mono text-lg font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(totals?.output_tokens ?? 0) }}</div></div>
      <div class="bg-white px-3 py-2.5 dark:bg-slate-900"><div class="text-[11px] font-medium uppercase tracking-wide text-slate-500">All tokens</div><div class="mt-1 font-mono text-lg font-semibold tabular-nums text-indigo-700 dark:text-indigo-300">{{ format(totals?.total_tokens ?? 0) }}</div></div>
    </div>

    <div v-if="!pending && !error && rows.length === 0" class="flex items-center gap-2 px-4 py-5 text-xs text-slate-500 dark:text-slate-400">
      <Activity class="h-4 w-4" aria-hidden="true" /> No model request usage has been recorded yet.
    </div>
    <div v-else-if="rows.length" class="max-h-[28rem] overflow-auto">
      <table class="w-full min-w-[760px] border-collapse text-left text-xs">
        <thead class="sticky top-0 z-10 bg-slate-50 text-[10px] uppercase tracking-wide text-slate-500 dark:bg-slate-950 dark:text-slate-400">
          <tr><th class="px-3 py-2 font-medium">Time</th><th v-if="scope === 'admin'" class="px-3 py-2 font-medium">User ID</th><th class="px-3 py-2 font-medium">Model</th><th class="px-3 py-2 font-medium">Agent</th><th class="px-3 py-2 text-right font-medium">Input</th><th class="px-3 py-2 text-right font-medium">Output</th><th class="px-3 py-2 text-right font-medium">Total</th></tr>
        </thead>
        <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
          <tr v-for="item in rows" :key="item.id" class="hover:bg-slate-50/70 dark:hover:bg-slate-800/50">
            <td class="whitespace-nowrap px-3 py-2.5 font-mono text-slate-500">{{ formatDate(item.timestamp) }}</td>
            <td v-if="scope === 'admin'" class="max-w-48 truncate px-3 py-2.5 font-mono text-slate-600 dark:text-slate-300" :title="item.user_id || 'Unknown user'">{{ item.user_id || 'Unknown user' }}</td>
            <td class="px-3 py-2.5"><div class="font-medium text-slate-800 dark:text-slate-200">{{ item.model_name }}</div><div class="text-[10px] text-slate-500">{{ item.model_provider }}</div></td>
            <td class="px-3 py-2.5 font-mono text-slate-600 dark:text-slate-300">{{ item.agent_id }}</td>
            <td class="px-3 py-2.5 text-right font-mono tabular-nums text-slate-600 dark:text-slate-300">{{ format(item.input_tokens) }}</td>
            <td class="px-3 py-2.5 text-right font-mono tabular-nums text-slate-600 dark:text-slate-300">{{ format(item.output_tokens) }}</td>
            <td class="px-3 py-2.5 text-right font-mono font-semibold tabular-nums text-slate-900 dark:text-white">{{ format(item.total_tokens) }}</td>
          </tr>
        </tbody>
      </table>
      <div class="flex items-center justify-between gap-3 border-t border-slate-100 px-3 py-2 dark:border-slate-800">
        <p v-if="olderError" class="text-xs text-rose-600 dark:text-rose-300" role="alert">Older requests could not be loaded.</p>
        <p v-else class="text-[11px] text-slate-500 dark:text-slate-400">{{ rows.length }} requests loaded; aggregate covers {{ format(totals?.request_count ?? 0) }}.</p>
        <button v-if="nextCursor" type="button" class="rounded border border-slate-200 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-60 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800" :disabled="loadingOlder" @click="loadOlder()">
          {{ loadingOlder ? 'Loading…' : 'Load older requests' }}
        </button>
      </div>
    </div>
  </section>
</template>
