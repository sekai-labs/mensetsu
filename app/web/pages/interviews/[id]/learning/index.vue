<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { calculateLearningProgress } from '~/features/interviews/utils/interview.util'
import AppButton from '~/components/ui/AppButton.vue'
import MarkdownRenderer from '~/components/ui/MarkdownRenderer.vue'

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
const currentPage = ref(1)
const pageSize = ref(6)

const progress = computed(() => {
  return calculateLearningProgress(learningUnits.value || [])
})

const totalPages = computed(() => {
  const total = (learningUnits.value || []).length
  return Math.max(1, Math.ceil(total / pageSize.value))
})

const paginatedUnits = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return (learningUnits.value || []).slice(start, start + pageSize.value)
})

const visiblePages = computed(() => {
  const total = totalPages.value
  const current = currentPage.value
  if (total <= 7) {
    return Array.from({ length: total }, (_, i) => i + 1)
  }
  const pages: (number | string)[] = [1]
  if (current > 3) pages.push('...')
  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let i = start; i <= end; i++) pages.push(i)
  if (current < total - 2) pages.push('...')
  pages.push(total)
  return pages
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

    <div v-else class="space-y-4">
      <div class="space-y-3">
        <div
          v-for="(unit, idx) in paginatedUnits"
          :key="unit.id"
          class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:border-indigo-500 transition-colors"
        >
          <div class="space-y-1.5 min-w-0 flex-1">
            <div class="flex items-center gap-2.5">
              <span class="font-mono text-xs font-bold text-indigo-600 dark:text-indigo-400">
                MODULE {{ String((currentPage - 1) * pageSize + idx + 1).padStart(2, '0') }}
              </span>
              <span
                v-if="unit.is_completed || unit.completed"
                class="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
              >
                COMPLETED
              </span>
              <span
                v-if="unit.chapters?.length"
                class="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-indigo-50 text-indigo-700 dark:bg-indigo-950/70 dark:text-indigo-300"
              >
                {{ unit.chapters.length }} SESSIONS UNDER-THE-HOOD
              </span>
            </div>

            <h3 class="text-sm font-semibold text-slate-900 dark:text-white">
              {{ unit.title }}
            </h3>

            <div v-if="unit.summary" class="text-xs text-slate-500 dark:text-slate-400">
              <MarkdownRenderer :content="unit.summary" dense />
            </div>

            <div class="flex flex-wrap items-center gap-3 text-xs text-slate-400 font-mono pt-1">
              <span>{{ unit.estimated_minutes }} mins total</span>
              <span v-if="unit.chapters?.length">• {{ unit.chapters.length }} technical sessions</span>
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
                {{ unit.is_completed || unit.completed ? 'Review Module →' : 'Start Module →' }}
              </AppButton>
            </NuxtLink>
          </div>
        </div>
      </div>

      <!-- Pagination Controls for Learning Plan -->
      <div v-if="totalPages > 1" class="flex items-center justify-between border-t border-slate-200 dark:border-slate-800 pt-4 text-xs">
        <div class="text-slate-500 font-mono">
          Page {{ currentPage }} of {{ totalPages }} ({{ learningUnits?.length || 0 }} modules)
        </div>
        <div class="flex items-center gap-1.5">
          <button
            type="button"
            class="px-3 py-1.5 rounded border border-slate-300 dark:border-slate-700 font-medium text-slate-700 dark:text-slate-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
            :disabled="currentPage <= 1"
            @click="currentPage--"
          >
            ← Previous
          </button>
          <template v-for="(p, pIdx) in visiblePages" :key="pIdx">
            <span
              v-if="p === '...'"
              class="w-7 h-7 flex items-center justify-center text-xs font-mono text-slate-400"
            >
              ...
            </span>
            <button
              v-else
              type="button"
              class="w-7 h-7 rounded text-xs font-mono font-bold transition-colors"
              :class="currentPage === p ? 'bg-indigo-600 text-white' : 'border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'"
              @click="currentPage = Number(p)"
            >
              {{ p }}
            </button>
          </template>
          <button
            type="button"
            class="px-3 py-1.5 rounded border border-slate-300 dark:border-slate-700 font-medium text-slate-700 dark:text-slate-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
            :disabled="currentPage >= totalPages"
            @click="currentPage++"
          >
            Next →
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
