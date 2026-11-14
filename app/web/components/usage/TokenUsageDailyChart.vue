<script setup lang="ts">
import { ref, computed } from 'vue'
import { BarChart3, AlertCircle, RefreshCw } from 'lucide-vue-next'
import type { DailyTokenUsageItem } from '~/models/metrics.model'
import type { TokenUsageTotals } from '@mensetsu/contracts'
import { formatDisplayDate } from '~/utils/date'

interface Props {
  items?: DailyTokenUsageItem[]
  totals?: TokenUsageTotals | null
  loading?: boolean
  error?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
  totals: null,
  loading: false,
  error: null,
})

const emit = defineEmits<{
  (e: 'retry'): void
}>()

const activeDayIndex = ref<number | null>(null)
const numberFormat = new Intl.NumberFormat()
const formatNumber = (num?: number) => numberFormat.format(num ?? 0)

// Format compact number for Y axis (e.g. 1.2k, 50k, 2.5M)
const formatCompact = (num: number): string => {
  if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`
  if (num >= 1_000) return `${(num / 1_000).toFixed(0)}k`
  return `${num}`
}

// Build a contiguous 30-day array leading up to today
const filledDays = computed(() => {
  const map = new Map<string, DailyTokenUsageItem>()
  for (const item of props.items) {
    map.set(item.date, item)
  }

  const days: DailyTokenUsageItem[] = []
  const today = new Date()

  for (let i = 29; i >= 0; i--) {
    const d = new Date(today)
    d.setDate(today.getDate() - i)
    const dateStr = d.toISOString().slice(0, 10)
    const found = map.get(dateStr)
    if (found) {
      days.push(found)
    } else {
      days.push({
        date: dateStr,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        request_count: 0,
      })
    }
  }

  return days
})

const maxTotalTokens = computed(() => {
  let max = 0
  for (const day of filledDays.value) {
    if (day.total_tokens > max) {
      max = day.total_tokens
    }
  }
  return max > 0 ? max : 1000
})

const activeDay = computed(() => {
  if (activeDayIndex.value === null) return null
  return filledDays.value[activeDayIndex.value] ?? null
})

// SVG dimensions & scales
const svgWidth = 800
const svgHeight = 220
const padLeft = 50
const padRight = 20
const padTop = 20
const padBottom = 35
const chartWidth = svgWidth - padLeft - padRight
const chartHeight = svgHeight - padTop - padBottom

const barWidth = computed(() => {
  const count = filledDays.value.length || 30
  return Math.max(8, (chartWidth / count) * 0.7)
})

const barX = (index: number) => {
  const count = filledDays.value.length || 30
  const slotWidth = chartWidth / count
  return padLeft + index * slotWidth + (slotWidth - barWidth.value) / 2
}

const barHeight = (value: number) => {
  if (maxTotalTokens.value <= 0) return 0
  return Math.max(value > 0 ? 2 : 0, (value / maxTotalTokens.value) * chartHeight)
}

const yGridLevels = computed(() => {
  const max = maxTotalTokens.value
  return [
    { label: formatCompact(max), y: padTop },
    { label: formatCompact(max * 0.75), y: padTop + chartHeight * 0.25 },
    { label: formatCompact(max * 0.5), y: padTop + chartHeight * 0.5 },
    { label: formatCompact(max * 0.25), y: padTop + chartHeight * 0.75 },
    { label: '0', y: padTop + chartHeight },
  ]
})

const handleKeydown = (e: KeyboardEvent, index: number) => {
  if (e.key === 'ArrowRight') {
    e.preventDefault()
    if (index < filledDays.value.length - 1) {
      activeDayIndex.value = index + 1
    }
  } else if (e.key === 'ArrowLeft') {
    e.preventDefault()
    if (index > 0) {
      activeDayIndex.value = index - 1
    }
  } else if (e.key === 'Escape') {
    activeDayIndex.value = null
  }
}
</script>

<template>
  <div
    class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900"
    role="region"
    aria-label="30-Day Token Usage Chart"
  >
    <header class="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-slate-100 dark:border-slate-800/80">
      <div class="flex items-center gap-2">
        <BarChart3 class="h-4 w-4 text-indigo-600 dark:text-indigo-400" aria-hidden="true" />
        <div>
          <h3 class="text-xs font-semibold uppercase tracking-[0.14em] text-slate-800 dark:text-slate-200">
            {{ $t('usage.chart.title') }}
          </h3>
          <p class="text-[11px] text-slate-500 dark:text-slate-400">
            {{ $t('usage.chart.subtitle') }}
          </p>
        </div>
      </div>

      <!-- Quick 30-Day Totals Pill -->
      <div v-if="totals" class="flex items-center gap-3 text-xs">
        <div class="hidden sm:flex items-center gap-1.5 font-mono">
          <span class="text-slate-500 dark:text-slate-400 text-[11px]">{{ $t('usage.chart.windowTotal') }}:</span>
          <span class="font-semibold text-indigo-600 dark:text-indigo-400">{{ formatNumber(totals.total_tokens) }}</span>
        </div>
        <div class="hidden md:flex items-center gap-1.5 font-mono">
          <span class="text-slate-500 dark:text-slate-400 text-[11px]">{{ $t('usage.chart.windowRequests') }}:</span>
          <span class="font-semibold text-slate-700 dark:text-slate-300">{{ formatNumber(totals.request_count) }}</span>
        </div>
      </div>
    </header>

    <!-- Loading Skeleton -->
    <div v-if="loading" class="py-10 text-center space-y-3" aria-live="polite">
      <div class="h-40 w-full animate-pulse rounded bg-slate-100 dark:bg-slate-800" />
      <p class="text-xs text-slate-400">{{ $t('usage.chart.loading') }}</p>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="my-6 flex items-center justify-between rounded-md border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-200"
      role="alert"
    >
      <div class="flex items-center gap-2">
        <AlertCircle class="h-4 w-4 shrink-0" aria-hidden="true" />
        <span>{{ error }}</span>
      </div>
      <button
        type="button"
        class="inline-flex items-center gap-1 rounded bg-white px-2 py-1 text-xs font-medium text-rose-700 shadow-sm hover:bg-rose-50 dark:bg-slate-800 dark:text-rose-300 dark:hover:bg-slate-700"
        @click="emit('retry')"
      >
        <RefreshCw class="h-3 w-3" aria-hidden="true" />
        <span>{{ $t('common.retry') }}</span>
      </button>
    </div>

    <!-- Active Day Tooltip Display Banner -->
    <div
      v-else
      class="mt-3 flex min-h-8 flex-wrap items-center justify-between gap-2 rounded bg-slate-50 px-3 py-1.5 text-xs text-slate-700 dark:bg-slate-850 dark:text-slate-200"
    >
      <template v-if="activeDay">
        <div class="flex items-center gap-2">
          <span class="font-semibold text-slate-900 dark:text-white font-mono text-[11px]">{{ formatDisplayDate(activeDay.date) }}</span>
          <span class="text-slate-300 dark:text-slate-700">•</span>
          <span class="font-mono text-indigo-600 dark:text-indigo-400 font-semibold">{{ formatNumber(activeDay.total_tokens) }} tokens</span>
          <span class="text-slate-400 dark:text-slate-500">({{ formatNumber(activeDay.request_count) }} reqs)</span>
        </div>
        <div class="flex items-center gap-3 text-[11px] font-mono text-slate-500 dark:text-slate-400">
          <span>In: {{ formatNumber(activeDay.input_tokens) }}</span>
          <span>Out: {{ formatNumber(activeDay.output_tokens) }}</span>
        </div>
      </template>
      <template v-else>
        <span class="text-[11px] text-slate-500 dark:text-slate-400">
          {{ $t('usage.chart.hoverHint') }}
        </span>
        <div class="flex items-center gap-3 text-[11px]">
          <div class="flex items-center gap-1.5">
            <span class="inline-block h-2.5 w-2.5 rounded-sm bg-indigo-500"></span>
            <span class="text-slate-500 dark:text-slate-400">{{ $t('usage.chart.totalTokens') }}</span>
          </div>
        </div>
      </template>
    </div>

    <!-- SVG Chart Container -->
    <div v-if="!loading && !error" class="mt-2 w-full overflow-x-auto">
      <svg
        :viewBox="`0 0 ${svgWidth} ${svgHeight}`"
        class="w-full h-auto min-w-[500px] select-none"
        role="img"
        aria-label="Daily token usage over the last 30 days"
      >
        <!-- Horizontal Grid Lines and Y-Axis Labels -->
        <g class="grid-lines" stroke="#e2e8f0" dark:stroke="#334155" stroke-dasharray="3,3">
          <g v-for="lvl in yGridLevels" :key="lvl.label">
            <line
              :x1="padLeft"
              :y1="lvl.y"
              :x2="svgWidth - padRight"
              :y2="lvl.y"
              stroke="currentColor"
              class="text-slate-200 dark:text-slate-800"
              stroke-width="1"
            />
            <text
              :x="padLeft - 8"
              :y="lvl.y + 3"
              text-anchor="end"
              class="fill-slate-400 dark:fill-slate-500 text-[10px] font-mono"
            >
              {{ lvl.label }}
            </text>
          </g>
        </g>

        <!-- Baseline -->
        <line
          :x1="padLeft"
          :y1="padTop + chartHeight"
          :x2="svgWidth - padRight"
          :y2="padTop + chartHeight"
          stroke="currentColor"
          class="text-slate-300 dark:text-slate-700"
          stroke-width="1"
        />

        <!-- Bars -->
        <g class="bars">
          <g
            v-for="(day, idx) in filledDays"
            :key="day.date"
            class="bar-group cursor-pointer focus:outline-none"
            tabindex="0"
            role="graphics-symbol"
            :aria-label="`${day.date}: ${day.total_tokens} tokens across ${day.request_count} requests`"
            @mouseenter="activeDayIndex = idx"
            @focus="activeDayIndex = idx"
            @mouseleave="activeDayIndex = null"
            @blur="activeDayIndex = null"
            @keydown="handleKeydown($event, idx)"
          >
            <!-- Background Hit Area -->
            <rect
              :x="barX(idx) - 2"
              :y="padTop"
              :width="barWidth + 4"
              :height="chartHeight"
              fill="transparent"
            />

            <!-- Daily Token Bar -->
            <rect
              :x="barX(idx)"
              :y="padTop + chartHeight - barHeight(day.total_tokens)"
              :width="barWidth"
              :height="barHeight(day.total_tokens)"
              rx="2"
              class="transition-colors duration-150"
              :class="activeDayIndex === idx
                ? 'fill-indigo-600 dark:fill-indigo-400'
                : (day.total_tokens > 0 ? 'fill-indigo-400/80 dark:fill-indigo-500/80 hover:fill-indigo-500' : 'fill-slate-200 dark:fill-slate-800')"
            />
          </g>
        </g>

        <!-- X-Axis Date Labels (shown every 5 days + last day) -->
        <g class="x-labels">
          <template v-for="(day, idx) in filledDays" :key="`lbl-${day.date}`">
            <text
              v-if="idx % 5 === 0 || idx === filledDays.length - 1"
              :x="barX(idx) + barWidth / 2"
              :y="svgHeight - 10"
              text-anchor="middle"
              class="fill-slate-400 dark:fill-slate-500 text-[10px] font-mono"
            >
              {{ day.date.slice(5) }}
            </text>
          </template>
        </g>
      </svg>
    </div>

    <!-- Screen-Reader Summary Table -->
    <div class="sr-only">
      <table>
        <caption>30-Day Token Usage Data</caption>
        <thead>
          <tr>
            <th scope="col">Date</th>
            <th scope="col">Total Tokens</th>
            <th scope="col">Input Tokens</th>
            <th scope="col">Output Tokens</th>
            <th scope="col">Requests</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="day in filledDays" :key="`sr-${day.date}`">
            <td>{{ day.date }}</td>
            <td>{{ day.total_tokens }}</td>
            <td>{{ day.input_tokens }}</td>
            <td>{{ day.output_tokens }}</td>
            <td>{{ day.request_count }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
