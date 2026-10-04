<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { calculateLearningProgress } from '../utils/interview.util'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceLearning } = useInterviewService()
const { data: learningUnits, pending } = await useAsyncData(
  `learning-plan-${workspaceId.value}`,
  () => getWorkspaceLearning(workspaceId.value)
)

const progressSummary = computed(() => {
  return calculateLearningProgress(learningUnits.value || [])
})
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2 mb-1">
            <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 font-mono">
              CURRICULUM
            </span>
            <span class="text-xs text-slate-400 font-medium">Estimated {{ progressSummary.totalMinutes }} Minutes Total</span>
          </div>
          <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
            Targeted Technical Course & Architectural Curriculum
          </h1>
          <p class="text-xs text-slate-500 mt-1 max-w-2xl">
            Prerequisite-ordered syllabus addressing candidate gaps with curated video lectures, failure domain case studies, and verification quizzes.
          </p>
        </div>

        <div class="flex flex-col items-end gap-1.5 shrink-0">
          <div class="text-xs font-mono text-slate-500">
            <strong>{{ progressSummary.completedCount }}</strong> of <strong>{{ progressSummary.totalCount }}</strong> completed
          </div>
          <div class="w-40 bg-slate-100 dark:bg-slate-800 rounded-full h-2 overflow-hidden border border-slate-200 dark:border-slate-700">
            <div
              class="bg-indigo-600 h-full rounded-full transition-all duration-300"
              :style="{ width: `${progressSummary.percentage}%` }"
            ></div>
          </div>
          <span class="text-[10px] font-bold text-indigo-600 dark:text-indigo-400 font-mono">{{ progressSummary.percentage }}% Complete</span>
        </div>
      </div>
    </div>

    <div v-if="pending" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-28 rounded-lg bg-slate-100 dark:bg-slate-800 animate-pulse"></div>
    </div>

    <div v-else-if="!learningUnits || learningUnits.length === 0" class="p-12 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg">
      No learning modules generated yet. Modules will be assembled once gap analysis completes.
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="(unit, idx) in learningUnits"
        :key="unit.id"
        class="p-6 rounded-lg border bg-white dark:bg-slate-900 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4 transition"
        :class="unit.completed
          ? 'border-emerald-200 dark:border-emerald-950/70 bg-emerald-50/20 dark:bg-emerald-950/10'
          : 'border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700'"
      >
        <div class="space-y-2 max-w-2xl">
          <div class="flex items-center gap-2.5">
            <span
              class="w-6 h-6 rounded-full font-mono text-xs font-bold flex items-center justify-center shrink-0"
              :class="unit.completed
                ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/60 dark:text-emerald-300'
                : 'bg-indigo-50 dark:bg-indigo-950 text-indigo-700 dark:text-indigo-400'"
            >
              {{ unit.completed ? '✓' : idx + 1 }}
            </span>
            <span class="text-xs font-mono uppercase tracking-wider text-slate-400">{{ unit.skill }}</span>
            <span class="text-xs px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 font-mono">
              {{ unit.estimated_minutes }} mins
            </span>
            <span
              v-if="unit.video_url"
              class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-amber-50 dark:bg-amber-950/60 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-800/50 flex items-center gap-1 font-mono"
            >
              ▶ VIDEO LECTURE
            </span>
          </div>

          <h2 class="text-base font-bold text-slate-900 dark:text-white">
            {{ unit.title }}
          </h2>

          <p class="text-xs text-slate-600 dark:text-slate-400 leading-relaxed">
            {{ unit.summary }}
          </p>

          <div v-if="unit.prerequisites && unit.prerequisites.length > 0" class="flex flex-wrap gap-1.5 pt-1">
            <span class="text-[11px] text-slate-400 font-medium">Prerequisites:</span>
            <span
              v-for="(prereq, pidx) in unit.prerequisites"
              :key="pidx"
              class="text-[11px] px-2 py-0.5 rounded bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 font-mono"
            >
              {{ prereq }}
            </span>
          </div>
        </div>

        <div class="flex flex-col sm:flex-row md:flex-col items-start md:items-end gap-2 shrink-0">
          <NuxtLink
            :to="`/interviews/${workspaceId}/learning/${unit.id}`"
            class="px-4 py-2 text-xs font-semibold rounded-md transition shadow-sm text-center w-full md:w-auto"
            :class="unit.completed
              ? 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-200'
              : 'bg-indigo-600 text-white hover:bg-indigo-500'"
          >
            {{ unit.completed ? 'Review Lesson' : 'Start Lesson →' }}
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
