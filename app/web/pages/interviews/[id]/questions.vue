<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { filterQuestions } from '~/features/interviews/utils/interview.util'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceQuestions } = useInterviewService()

const { data: questions, pending, error: questionsError, refresh: refreshQuestions } = await useAsyncData(
  `questions-${workspaceId.value}`,
  () => getWorkspaceQuestions(workspaceId.value)
)

const searchQuery = ref('')
const categoryFilter = ref('all')
const difficultyFilter = ref('all')
const expandedQuestionId = ref<string | null>(null)
const currentPage = ref(1)
const pageSize = ref(5)

const toggleExpand = (id: string) => {
  expandedQuestionId.value = expandedQuestionId.value === id ? null : id
}
const availableCategories = computed(() => {
  const set = new Set<string>()
  for (const q of questions.value || []) {
    if (q.category) set.add(q.category)
  }
  return Array.from(set).sort()
})

const filteredList = computed(() => {
  return filterQuestions(questions.value || [], {
    searchQuery: searchQuery.value,
    category: categoryFilter.value,
    difficulty: difficultyFilter.value,
  })
})
const totalPages = computed(() => Math.max(1, Math.ceil(filteredList.value.length / pageSize.value)))

const paginatedList = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredList.value.slice(start, start + pageSize.value)
})

watch([searchQuery, categoryFilter, difficultyFilter], () => {
  currentPage.value = 1
})
</script>

<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>INTERVIEW QUESTION BOOK</span>
          <span>•</span>
          <span>CONTEXT GROUNDED</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Question Bank & Answer Outlines
        </h1>
      </div>

      <div class="text-xs font-mono text-slate-500">
        {{ filteredList.length }} questions available
      </div>
    </div>


    <div class="flex flex-col sm:flex-row gap-3 items-center justify-between">
      <div class="w-full sm:w-72">
        <AppInput
          v-model="searchQuery"
          placeholder="Filter questions or skills..."
        />
      </div>

      <div class="flex flex-wrap items-center gap-1.5 self-start sm:self-auto text-xs">
        <button
          v-for="diff in ['all', 'easy', 'medium', 'hard']"
          :key="diff"
          type="button"
          class="px-2.5 py-1 rounded-md capitalize font-medium transition-colors"
          :class="difficultyFilter === diff
            ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
            : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'"
          @click="difficultyFilter = diff"
        >
          {{ diff }}
        </button>
      </div>
    </div>

    <div v-if="availableCategories.length > 1" class="flex flex-wrap items-center gap-1.5 text-xs pb-1">
      <span class="text-slate-400 font-mono text-[11px] mr-1">Category:</span>
      <button
        type="button"
        class="px-2.5 py-0.5 rounded-full capitalize font-medium transition-colors border text-[11px]"
        :class="categoryFilter === 'all'
          ? 'bg-indigo-600 text-white border-indigo-600'
          : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800'"
        @click="categoryFilter = 'all'"
      >
        All Categories ({{ questions?.length || 0 }})
      </button>
      <button
        v-for="cat in availableCategories"
        :key="cat"
        type="button"
        class="px-2.5 py-0.5 rounded-full capitalize font-medium transition-colors border text-[11px]"
        :class="categoryFilter === cat
          ? 'bg-indigo-600 text-white border-indigo-600'
          : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800'"
        @click="categoryFilter = cat"
      >
        {{ cat.replace(/_/g, ' ') }} ({{ (questions || []).filter(q => q.category === cat).length }})
      </button>
    </div>

    <div v-if="questionsError" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load interview questions</div>
      <p class="text-rose-600 dark:text-rose-400">{{ questionsError.message || String(questionsError) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refreshQuestions()"
      >
        Retry Question Bank Fetch
      </button>
    </div>

    <div v-else-if="pending" class="space-y-3">
      <div class="h-20 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-20 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-20 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <div v-else-if="!filteredList.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No questions found matching criteria.
    </div>
    <div v-else class="space-y-3">
      <div
        v-for="q in paginatedList"
        :key="q.id"
        class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm overflow-hidden transition-colors"
      >
        <div
          class="p-4 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-850 flex items-start justify-between gap-4"
          @click="toggleExpand(q.id)"
        >
          <div class="space-y-1.5 min-w-0">
            <div class="flex items-center gap-2">
              <span
                class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded font-bold"
                :class="{
                  'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300': q.difficulty === 'easy',
                  'bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300': q.difficulty === 'medium',
                  'bg-rose-50 text-rose-700 dark:bg-rose-950 dark:text-rose-300': q.difficulty === 'hard',
                }"
              >
                {{ q.difficulty }}
              </span>
              <span class="text-[11px] font-mono text-slate-400 uppercase">
                {{ q.category.replace(/_/g, ' ') }}
              </span>
            </div>

            <h3 class="text-sm font-semibold text-slate-900 dark:text-white leading-snug">
              {{ q.question }}
            </h3>

            <div v-if="q.skills_tested?.length" class="flex flex-wrap gap-1.5 pt-1">
              <span
                v-for="(sk, idx) in q.skills_tested"
                :key="idx"
                class="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-[10px] font-mono text-slate-600 dark:text-slate-300"
              >
                {{ sk }}
              </span>
            </div>
          </div>

          <button
            type="button"
            class="text-xs text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 shrink-0 font-mono"
          >
            {{ expandedQuestionId === q.id ? 'Collapse ▲' : 'Inspect ▼' }}
          </button>
        </div>


        <div
          v-if="expandedQuestionId === q.id"
          class="p-5 border-t border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-850/40 space-y-4 text-xs leading-relaxed"
        >
          <div v-if="q.interviewer_intent" class="space-y-1">
            <span class="text-[10px] font-mono uppercase font-bold text-slate-400">Interviewer Intent & Focus</span>
            <p class="text-slate-700 dark:text-slate-300 m-0">{{ q.interviewer_intent }}</p>
          </div>

          <div v-if="q.answer_outline" class="space-y-1.5">
            <span class="text-[10px] font-mono uppercase font-bold text-slate-400">Outline Answer</span>
            <div v-if="Array.isArray(q.answer_outline)" class="space-y-1">
              <div v-for="(pt, idx) in q.answer_outline" :key="idx" class="flex items-start gap-2 text-slate-700 dark:text-slate-300">
                <span class="text-indigo-600 dark:text-indigo-400 font-bold">•</span>
                <span>{{ pt }}</span>
              </div>
            </div>
            <p v-else class="text-slate-700 dark:text-slate-300 m-0">{{ q.answer_outline }}</p>
          </div>

          <div v-if="q.strong_answer_rubric?.length" class="space-y-1.5 pt-2 border-t border-slate-200 dark:border-slate-800">
            <span class="text-[10px] font-mono uppercase font-bold text-emerald-600 dark:text-emerald-400">Strong Answer Criteria</span>
            <ul class="space-y-1 list-disc pl-5 text-slate-700 dark:text-slate-300">
              <li v-for="(ru, idx) in q.strong_answer_rubric" :key="idx">{{ ru }}</li>
            </ul>
          </div>

          <div v-if="q.common_weaknesses?.length" class="space-y-1.5">
            <span class="text-[10px] font-mono uppercase font-bold text-rose-600 dark:text-rose-400">Common Pitfalls & Weaknesses</span>
            <ul class="space-y-1 list-disc pl-5 text-slate-700 dark:text-slate-300">
              <li v-for="(cw, idx) in q.common_weaknesses" :key="idx">{{ cw }}</li>
            </ul>
          </div>

          <div v-if="q.follow_up_questions?.length" class="space-y-1.5 pt-2 border-t border-slate-200 dark:border-slate-800">
            <span class="text-[10px] font-mono uppercase font-bold text-slate-400">Anticipated Follow-Up Probes</span>
            <ul class="space-y-1 list-disc pl-5 text-slate-600 dark:text-slate-400">
              <li v-for="(fu, idx) in q.follow_up_questions" :key="idx">{{ fu }}</li>
            </ul>
          </div>
        </div>
      </div>


      <div v-if="totalPages > 1" class="flex items-center justify-between border-t border-slate-200 dark:border-slate-800 pt-4 text-xs">
        <div class="text-slate-500 font-mono">
          Page {{ currentPage }} of {{ totalPages }} ({{ filteredList.length }} items)
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
          <button
            v-for="p in totalPages"
            :key="p"
            type="button"
            class="w-7 h-7 rounded text-xs font-mono font-bold transition-colors"
            :class="currentPage === p ? 'bg-indigo-600 text-white' : 'border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'"
            @click="currentPage = p"
          >
            {{ p }}
          </button>
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
