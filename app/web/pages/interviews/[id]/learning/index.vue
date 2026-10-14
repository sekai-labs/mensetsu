<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { calculateLearningProgress } from '~/features/interviews/utils/interview.util'
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceLearning, toggleLearningUnitCompleted } = useInterviewService()

const { data: learningUnits, refresh } = await useAsyncData(
  `learning-plan-${workspaceId.value}`,
  () => getWorkspaceLearning(workspaceId.value)
)

const progress = computed(() => {
  return calculateLearningProgress(learningUnits.value || [])
})

const handleToggle = async (lessonId: string) => {
  await toggleLearningUnitCompleted(workspaceId.value, lessonId)
  await refresh()
}
</script>

<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>PREREQUISITE-ORDERED SYLLABUS</span>
          <span>•</span>
          <span>CURATED CURRICULUM</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Learning Plan & Technical Lessons
        </h1>
      </div>

      <div class="flex items-center gap-3">
        <div class="text-right">
          <div class="text-[10px] font-mono uppercase text-slate-400">Completion</div>
          <div class="text-2xl font-bold font-mono text-emerald-600 dark:text-emerald-400 tabular-nums">
            {{ progress.percentage }}%
          </div>
        </div>
      </div>
    </div>


    <div v-if="!learningUnits?.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      Synthesizing customized syllabus for target role...
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="(unit, idx) in learningUnits"
        :key="unit.id"
        class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:border-indigo-500 transition-colors"
      >
        <div class="space-y-1.5 min-w-0">
          <div class="flex items-center gap-2.5">
            <span class="font-mono text-xs font-bold text-indigo-600 dark:text-indigo-400">
              MODULE {{ String(idx + 1).padStart(2, '0') }}
            </span>
            <span
              v-if="unit.is_completed || unit.completed"
              class="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
            >
              COMPLETED
            </span>
          </div>

          <h3 class="text-sm font-semibold text-slate-900 dark:text-white">
            {{ unit.title }}
          </h3>

          <p v-if="unit.summary" class="text-xs text-slate-500 line-clamp-2 m-0">
            {{ unit.summary }}
          </p>

          <div class="flex items-center gap-3 text-xs text-slate-400 font-mono pt-1">
            <span>{{ unit.estimated_minutes }} mins</span>
            <span v-if="unit.prerequisites?.length">• Prereqs: {{ unit.prerequisites.join(', ') }}</span>
          </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <button
            type="button"
            class="px-3 py-1.5 text-xs font-medium rounded border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 transition-colors"
            @click="handleToggle(unit.id)"
          >
            {{ unit.is_completed || unit.completed ? 'Mark Incomplete' : 'Mark Completed' }}
          </button>

          <NuxtLink :to="`/interviews/${workspaceId}/learning/${unit.id.replace(/^learning_unit:/, '').replace(/`/g, '')}`">
            <AppButton size="xs">
              {{ unit.is_completed || unit.completed ? 'Review Lesson →' : 'Start Lesson →' }}
            </AppButton>
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
