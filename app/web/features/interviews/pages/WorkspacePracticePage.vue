<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { InterviewQuestionResponse } from '../models/interview.model'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceQuestions } = useInterviewService()
const { data: questions, pending } = await useAsyncData(
  `practice-questions-${workspaceId.value}`,
  () => getWorkspaceQuestions(workspaceId.value)
)

const focusId = computed(() => route.query.focus as string | undefined)

const currentQuestionIndex = ref(0)
const showAnswerRubric = ref(false)
const userNotes = ref('')
const completedCount = ref(0)

const activeQuestions = computed<InterviewQuestionResponse[]>(() => {
  if (!questions.value) return []
  if (focusId.value) {
    const matched = questions.value.find((q) => q.id === focusId.value)
    if (matched) {
      return [matched, ...questions.value.filter((q) => q.id !== focusId.value)]
    }
  }
  return questions.value
})

const currentQuestion = computed<InterviewQuestionResponse | null>(() => {
  return activeQuestions.value[currentQuestionIndex.value] || null
})

const handleNextQuestion = () => {
  showAnswerRubric.value = false
  userNotes.value = ''
  completedCount.value += 1
  if (currentQuestionIndex.value < activeQuestions.value.length - 1) {
    currentQuestionIndex.value += 1
  } else {
    currentQuestionIndex.value = 0
  }
}
</script>

<template>
  <div class="space-y-6 max-w-4xl mx-auto">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interactive Interview Practice Session
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Simulate real interview conditions. Formulate your answer first before revealing expected rubrics.
        </p>
      </div>

      <div class="flex items-center gap-3">
        <span class="text-xs font-mono text-slate-400">Drilled: {{ completedCount }} questions</span>
        <NuxtLink
          :to="`/interviews/${workspaceId}/questions`"
          class="text-xs font-semibold text-indigo-600 hover:text-indigo-500"
        >
          View Book →
        </NuxtLink>
      </div>
    </div>

    <div v-if="pending" class="h-64 rounded-lg bg-slate-100 dark:bg-slate-800 animate-pulse"></div>

    <div v-else-if="!currentQuestion" class="p-12 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg">
      No interview questions available to practice. Complete the workspace research phase first.
    </div>

    <div v-else class="space-y-6">
      <div class="p-6 sm:p-8 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-5">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">
              {{ currentQuestion.category?.replace(/_/g, ' ') }}
            </span>
            <span class="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-indigo-50 text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-400">
              {{ currentQuestion.difficulty }}
            </span>
            <span v-if="currentQuestion.skills_tested?.length" class="text-xs text-slate-400 font-mono">
              Skills: {{ currentQuestion.skills_tested.join(', ') }}
            </span>
          </div>

          <span class="text-xs font-mono text-slate-400 self-end sm:self-auto">
            {{ currentQuestionIndex + 1 }} / {{ activeQuestions.length }}
          </span>
        </div>

        <h2 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white leading-snug">
          "{{ currentQuestion.question }}"
        </h2>

        <div class="space-y-2 pt-2">
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-500">
            Scratchpad / Answer Outline Notes
          </label>
          <textarea
            v-model="userNotes"
            rows="4"
            placeholder="Structure your thoughts here: Situation/Architecture, Tradeoffs, Decision, Failure handling..."
            class="w-full px-3.5 py-2.5 rounded-md border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 text-xs text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500 font-mono"
          ></textarea>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-slate-100 dark:border-slate-800">
          <button
            @click="showAnswerRubric = !showAnswerRubric"
            class="px-4 py-2 text-xs font-semibold rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition"
          >
            {{ showAnswerRubric ? 'Hide Rubric' : 'Reveal Answer Rubric & Pitfalls' }}
          </button>

          <button
            @click="handleNextQuestion"
            class="px-5 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
          >
            Next Question →
          </button>
        </div>
      </div>

      <div
        v-if="showAnswerRubric"
        class="p-6 rounded-lg border border-indigo-200 dark:border-indigo-900/60 bg-indigo-50/20 dark:bg-indigo-950/20 shadow-sm space-y-5 animate-fade-in"
      >
        <div class="flex items-center justify-between border-b border-indigo-100 dark:border-indigo-900/40 pb-3">
          <h3 class="text-sm font-bold text-slate-900 dark:text-white">
            Evaluation Rubric & Follow-Up Anticipation
          </h3>
          <span class="text-[11px] font-mono text-indigo-600 dark:text-indigo-400">STAR / Senior Criteria</span>
        </div>

        <div v-if="currentQuestion.interviewer_intent">
          <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
            Interviewer Intent & Assessment Focus:
          </span>
          <p class="text-xs text-slate-600 dark:text-slate-400 mt-1 leading-relaxed">
            {{ currentQuestion.interviewer_intent }}
          </p>
        </div>

        <div v-if="currentQuestion.answer_outline?.length">
          <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
            Expected Answer Architecture:
          </span>
          <ul class="mt-1 space-y-1.5 text-xs text-slate-600 dark:text-slate-400">
            <li v-for="(point, idx) in currentQuestion.answer_outline" :key="idx" class="flex items-start gap-2">
              <span class="text-indigo-600 font-bold">•</span>
              <span>{{ point }}</span>
            </li>
          </ul>
        </div>

        <div v-if="currentQuestion.strong_answer_rubric?.length">
          <span class="font-semibold text-emerald-700 dark:text-emerald-400 uppercase tracking-wider text-[10px]">
            Passing / Bar-Raiser Criteria:
          </span>
          <ul class="mt-1 space-y-1 text-xs text-slate-600 dark:text-slate-400">
            <li v-for="(rubric, idx) in currentQuestion.strong_answer_rubric" :key="idx" class="flex items-start gap-2">
              <span class="text-emerald-500 font-bold">✓</span>
              <span>{{ rubric }}</span>
            </li>
          </ul>
        </div>

        <div v-if="currentQuestion.common_weaknesses?.length">
          <span class="font-semibold text-rose-700 dark:text-rose-400 uppercase tracking-wider text-[10px]">
            Candidate Traps & Pitfalls:
          </span>
          <ul class="mt-1 space-y-1 text-xs text-slate-600 dark:text-slate-400">
            <li v-for="(weak, idx) in currentQuestion.common_weaknesses" :key="idx" class="flex items-start gap-2">
              <span class="text-rose-500 font-bold">✗</span>
              <span>{{ weak }}</span>
            </li>
          </ul>
        </div>

        <div v-if="currentQuestion.follow_up_questions?.length">
          <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
            Follow-Up Probes To Be Ready For:
          </span>
          <ul class="mt-1 space-y-1 text-xs text-slate-600 dark:text-slate-400 italic">
            <li v-for="(f, idx) in currentQuestion.follow_up_questions" :key="idx" class="flex items-start gap-2">
              <span class="text-amber-500 font-bold">↳</span>
              <span>"{{ f }}"</span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
