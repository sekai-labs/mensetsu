<script setup lang="ts">
import type { InterviewQuestionResponse } from '../models/interview.model'

defineProps<{
  question: InterviewQuestionResponse
  isExpanded: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
}>()
</script>

<template>
  <div class="rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm overflow-hidden transition">
    <div
      @click="emit('toggle')"
      class="p-4 sm:p-5 flex items-start justify-between gap-4 cursor-pointer hover:bg-slate-50/50 dark:hover:bg-slate-850/50"
    >
      <div class="space-y-1.5 flex-1">
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">
            {{ question.category ? question.category.replace(/_/g, ' ') : 'General' }}
          </span>
          <span
            class="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded"
            :class="{
              'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-400': question.difficulty === 'easy' || question.difficulty === 'junior',
              'bg-blue-50 text-blue-700 dark:bg-blue-950/50 dark:text-blue-400': question.difficulty === 'medium' || question.difficulty === 'mid',
              'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-400': question.difficulty === 'hard' || question.difficulty === 'senior',
              'bg-purple-50 text-purple-700 dark:bg-purple-950/50 dark:text-purple-400': question.difficulty === 'staff',
            }"
          >
            {{ question.difficulty }}
          </span>
          <span v-if="question.skills_tested?.length" class="text-xs text-slate-400 font-mono">
            Skills: {{ question.skills_tested.join(', ') }}
          </span>
          <span v-else-if="question.skill_tested" class="text-xs text-slate-400 font-mono">
            Skill: {{ question.skill_tested }}
          </span>
        </div>

        <h3 class="text-sm font-semibold text-slate-900 dark:text-white pt-1">
          {{ question.question }}
        </h3>
      </div>

      <div class="text-slate-400 text-sm font-mono pt-1">
        {{ isExpanded ? '−' : '+' }}
      </div>
    </div>

    <div
      v-if="isExpanded"
      class="p-5 border-t border-slate-100 dark:border-slate-800 bg-slate-50/40 dark:bg-slate-950/30 space-y-4 text-xs"
    >
      <div v-if="question.interviewer_intent">
        <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
          Interviewer Intent & Assessment Focus:
        </span>
        <p class="text-slate-600 dark:text-slate-400 mt-1 leading-relaxed">
          {{ question.interviewer_intent }}
        </p>
      </div>

      <div v-if="question.answer_outline?.length">
        <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
          Ideal Answer Structure:
        </span>
        <ul class="mt-1 space-y-1 text-slate-600 dark:text-slate-400">
          <li v-for="(point, pidx) in question.answer_outline" :key="pidx" class="flex items-start gap-2">
            <span class="text-indigo-600 font-bold">•</span>
            <span>{{ point }}</span>
          </li>
        </ul>
      </div>

      <div v-if="question.strong_answer_rubric?.length">
        <span class="font-semibold text-emerald-700 dark:text-emerald-400 uppercase tracking-wider text-[10px]">
          Passing / Bar-Raiser Criteria:
        </span>
        <ul class="mt-1 space-y-1 text-emerald-800 dark:text-emerald-300">
          <li v-for="(r, ridx) in question.strong_answer_rubric" :key="ridx" class="flex items-start gap-2">
            <span>✓</span>
            <span>{{ r }}</span>
          </li>
        </ul>
      </div>

      <div v-if="question.common_weaknesses?.length">
        <span class="font-semibold text-rose-700 dark:text-rose-400 uppercase tracking-wider text-[10px]">
          Common Traps & Red Flags:
        </span>
        <ul class="mt-1 space-y-1 text-rose-800 dark:text-rose-300">
          <li v-for="(w, widx) in question.common_weaknesses" :key="widx" class="flex items-start gap-2">
            <span>✕</span>
            <span>{{ w }}</span>
          </li>
        </ul>
      </div>

      <div v-if="question.follow_up_questions?.length" class="pt-2 border-t border-slate-200/50 dark:border-slate-800/50">
        <span class="font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider text-[10px]">
          Probable Deep-Dive Follow-ups:
        </span>
        <ul class="mt-1 space-y-1 text-slate-500 italic">
          <li v-for="(f, fidx) in question.follow_up_questions" :key="fidx" class="flex items-start gap-2">
            <span>↪</span>
            <span>"{{ f }}"</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
