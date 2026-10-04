<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { filterQuestions } from '../utils/interview.util'
import QuestionBookItem from '../components/QuestionBookItem.vue'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceQuestions } = useInterviewService()
const { data: questions, pending } = await useAsyncData(
  `questions-book-${workspaceId.value}`,
  () => getWorkspaceQuestions(workspaceId.value)
)

const searchQuery = ref('')
const selectedDifficulty = ref<'all' | 'junior' | 'mid' | 'senior' | 'staff'>('all')
const selectedCategory = ref('all')
const expandedQuestionId = ref<string | null>(null)

const toggleExpand = (id: string) => {
  expandedQuestionId.value = expandedQuestionId.value === id ? null : id
}

const categories = computed(() => {
  if (!questions.value) return ['all']
  const set = new Set(questions.value.map((q) => q.category))
  return ['all', ...Array.from(set)]
})

const filteredQuestions = computed(() => {
  return filterQuestions(questions.value || [], {
    searchQuery: searchQuery.value,
    category: selectedCategory.value,
    difficulty: selectedDifficulty.value,
  })
})
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Targeted Interview Question Book
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Curated technical rubrics, expected answer architectures, follow-up traps, and candidate pitfalls.
        </p>
      </div>

      <NuxtLink
        :to="`/interviews/${workspaceId}/practice`"
        class="inline-flex items-center px-4 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm self-start md:self-auto"
      >
        Launch Interactive Practice →
      </NuxtLink>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 w-full sm:w-auto">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search question or skill..."
          class="px-3 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-full sm:w-64"
        />
        <select
          v-model="selectedCategory"
          class="px-2.5 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-700 dark:text-slate-300 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-full sm:w-auto"
        >
          <option v-for="cat in categories" :key="cat" :value="cat">
            {{ cat === 'all' ? 'All Categories' : cat.replace(/_/g, ' ') }}
          </option>
        </select>
      </div>

      <div class="flex items-center gap-1 overflow-x-auto scrollbar-none pb-1 sm:pb-0">
        <button
          v-for="diff in ['all', 'junior', 'mid', 'senior', 'staff'] as const"
          :key="diff"
          @click="selectedDifficulty = diff"
          class="px-2.5 py-1 rounded text-xs font-semibold uppercase tracking-wider transition shrink-0"
          :class="selectedDifficulty === diff
            ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
            : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700'"
        >
          {{ diff }}
        </button>
      </div>
    </div>

    <div v-if="pending" class="space-y-4">
      <div v-for="i in 4" :key="i" class="h-20 bg-slate-100 dark:bg-slate-800 rounded-lg animate-pulse"></div>
    </div>

    <div v-else-if="filteredQuestions.length === 0" class="p-12 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg">
      No interview questions match your filter criteria.
    </div>

    <div v-else class="space-y-3">
      <QuestionBookItem
        v-for="q in filteredQuestions"
        :key="q.id"
        :question="q"
        :is-expanded="expandedQuestionId === q.id"
        @toggle="toggleExpand(q.id)"
      />
    </div>
  </div>
</template>
