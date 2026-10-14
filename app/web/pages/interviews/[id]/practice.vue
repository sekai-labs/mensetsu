<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceQuestions } = useInterviewService()

const { data: questions } = await useAsyncData(
  `practice-questions-${workspaceId.value}`,
  () => getWorkspaceQuestions(workspaceId.value)
)

const currentIndex = ref<number>(0)
const candidateNotes = ref<string>('Initial design thought: Anycast DNS ingress into local PoP edge proxies with NVMe caching layer.')
const isTimerRunning = ref<boolean>(false)
const secondsElapsed = ref<number>(0)
let timerInterval: NodeJS.Timeout | null = null

const activeQuestion = computed(() => {
  const list = questions.value || []
  return list[currentIndex.value] || null
})

const toggleTimer = () => {
  if (isTimerRunning.value) {
    if (timerInterval) clearInterval(timerInterval)
    isTimerRunning.value = false
  } else {
    isTimerRunning.value = true
    timerInterval = setInterval(() => {
      secondsElapsed.value++
    }, 1000)
  }
}

const formattedTime = computed(() => {
  const mins = Math.floor(secondsElapsed.value / 60)
  const secs = secondsElapsed.value % 60
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
})

const nextQuestion = () => {
  const list = questions.value || []
  if (currentIndex.value < list.length - 1) {
    currentIndex.value++
  }
}

const prevQuestion = () => {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

onUnmounted(() => {
  if (timerInterval) clearInterval(timerInterval)
})
</script>

<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>PRACTICE COCKPIT</span>
          <span>•</span>
          <span>FOCUSED SIMULATION</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interactive Interview Practice & Scratchpad
        </h1>
      </div>


      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-xs font-mono">
          <span class="w-2 h-2 rounded-full" :class="isTimerRunning ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'"></span>
          <span class="font-bold text-slate-800 dark:text-slate-200 tabular-nums">{{ formattedTime }}</span>
          <button
            type="button"
            class="text-indigo-600 dark:text-indigo-400 hover:underline font-medium ml-1"
            @click="toggleTimer"
          >
            {{ isTimerRunning ? 'Pause' : 'Start' }}
          </button>
        </div>

        <div class="flex items-center gap-1">
          <button
            type="button"
            class="px-2.5 py-1.5 rounded border border-slate-300 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-40"
            :disabled="currentIndex <= 0"
            @click="prevQuestion"
          >
            ←
          </button>
          <span class="text-xs font-mono px-2 text-slate-500">
            {{ currentIndex + 1 }} / {{ questions?.length || 1 }}
          </span>
          <button
            type="button"
            class="px-2.5 py-1.5 rounded border border-slate-300 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-40"
            :disabled="!questions || currentIndex >= questions.length - 1"
            @click="nextQuestion"
          >
            →
          </button>
        </div>
      </div>
    </div>


    <div v-if="activeQuestion" class="space-y-6">
      <div class="p-6 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm space-y-3">
        <div class="flex items-center gap-2">
          <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-700 dark:text-indigo-300 font-bold">
            {{ activeQuestion.category.replace(/_/g, ' ') }}
          </span>
          <span class="text-[10px] font-mono uppercase text-slate-400">
            Difficulty: {{ activeQuestion.difficulty }}
          </span>
        </div>

        <h2 class="text-base font-semibold text-slate-900 dark:text-white leading-snug">
          {{ activeQuestion.question }}
        </h2>

        <p v-if="activeQuestion.interviewer_intent" class="text-xs text-slate-500 m-0 leading-relaxed">
          <span class="font-medium text-slate-700 dark:text-slate-300">Target Invariants: </span>
          {{ activeQuestion.interviewer_intent }}
        </p>
      </div>


      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-xs font-mono font-bold uppercase tracking-wider text-slate-500">
            Candidate System Design Scratchpad & Notes
          </label>
          <span class="text-[11px] text-slate-400">Auto-saved in memory</span>
        </div>

        <textarea
          v-model="candidateNotes"
          rows="10"
          placeholder="Jot down architectural nodes, API request/response contracts, consensus protocols, and latency budgets..."
          class="block w-full rounded-md border text-xs p-4 font-mono leading-relaxed bg-white dark:bg-slate-900 border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        ></textarea>
      </div>
    </div>
  </div>
</template>
