<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import MarkdownRenderer from '~/components/ui/MarkdownRenderer.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const lessonId = computed(() => String(route.params.lessonId))
const { getWorkspaceLearning, toggleLearningUnitCompleted } = useInterviewService()

const { data: units, refresh } = await useAsyncData(
  `learning-units-${workspaceId.value}`,
  () => getWorkspaceLearning(workspaceId.value)
)

const currentLesson = computed(() => {
  return (units.value || []).find((u) => {
    const cleanUId = u.id.replace(/^learning_unit:/, '').replace(/`/g, '')
    return cleanUId === lessonId.value || u.id === lessonId.value
  })
})

const activeSessionIndex = ref<number>(0)

const handleToggle = async () => {
  if (currentLesson.value) {
    await toggleLearningUnitCompleted(workspaceId.value, currentLesson.value.id)
    await refresh()
  }
}
</script>

<template>
  <div class="space-y-6 max-w-3xl">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <NuxtLink
          :to="`/interviews/${workspaceId}/learning`"
          class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium inline-block mb-1.5"
        >
          ← Back to Syllabus
        </NuxtLink>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ currentLesson?.title || 'Lesson Overview' }}
        </h1>
        <div class="flex items-center gap-3 text-xs text-slate-500 font-mono mt-1">
          <span>{{ currentLesson?.estimated_minutes || 30 }} mins</span>
          <span v-if="currentLesson?.skill">• Skill: {{ currentLesson.skill }}</span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="px-3 py-1.5 text-xs font-medium rounded border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 transition-colors"
          @click="handleToggle"
        >
          {{ currentLesson?.is_completed || currentLesson?.completed ? 'Mark Incomplete' : 'Mark Completed' }}
        </button>
      </div>
    </div>


    <article class="p-6 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm space-y-6 text-sm leading-relaxed text-slate-800 dark:text-slate-200">
      <div v-if="currentLesson?.summary" class="p-4 bg-slate-50 dark:bg-slate-850 rounded border-l-2 border-indigo-600 text-xs italic text-slate-600 dark:text-slate-400">
        <MarkdownRenderer :content="currentLesson.summary" dense />
      </div>

      <div class="space-y-4">
        <MarkdownRenderer
          :content="currentLesson?.content_markdown || currentLesson?.deep_dive_markdown || 'Synthesizing technical deep dive for this module. Focus on core architectural invariants, edge failure cases, and operational mitigations.'"
        />
      </div>


      <!-- Under-the-hood In-depth Sessions -->
      <div v-if="currentLesson?.chapters?.length" class="space-y-4 pt-4 border-t border-slate-200 dark:border-slate-800">
        <div class="flex items-center justify-between">
          <div class="text-xs font-mono font-bold uppercase tracking-wider text-indigo-600 dark:text-indigo-400">
            Under-the-Hood Technical Sessions ({{ currentLesson.chapters.length }})
          </div>
          <div class="text-[11px] font-mono text-slate-400">
            Deep Internals & Invariants
          </div>
        </div>

        <!-- Session Tabs -->
        <div class="flex flex-wrap gap-1.5 p-1 bg-slate-100 dark:bg-slate-800/60 rounded-lg">
          <button
            v-for="(session, sIdx) in currentLesson.chapters"
            :key="session.id || sIdx"
            type="button"
            class="px-3 py-1.5 rounded-md text-xs font-medium transition-colors text-left"
            :class="activeSessionIndex === sIdx
              ? 'bg-white dark:bg-slate-900 text-indigo-600 dark:text-indigo-400 shadow-sm font-semibold'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            @click="activeSessionIndex = sIdx"
          >
            <span class="font-mono text-[10px] opacity-70 mr-1.5">S{{ sIdx + 1 }}.</span>
            <span>{{ session.title }}</span>
          </button>
        </div>

        <!-- Active Session Content -->
        <div
          v-if="currentLesson.chapters[activeSessionIndex]"
          class="p-4 rounded-lg bg-slate-50 dark:bg-slate-850/80 border border-slate-200 dark:border-slate-750 space-y-3"
        >
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-200 dark:border-slate-700/60 pb-2.5">
            <h4 class="text-sm font-bold text-slate-900 dark:text-white">
              {{ currentLesson.chapters[activeSessionIndex].title }}
            </h4>
            <div class="flex items-center gap-2 text-xs font-mono text-slate-500">
              <span v-if="currentLesson.chapters[activeSessionIndex].focus_area" class="px-2 py-0.5 rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-700 dark:text-indigo-300 font-semibold">
                {{ currentLesson.chapters[activeSessionIndex].focus_area }}
              </span>
              <span v-if="currentLesson.chapters[activeSessionIndex].timestamp">
                {{ currentLesson.chapters[activeSessionIndex].timestamp }}
              </span>
            </div>
          </div>

          <div v-if="currentLesson.chapters[activeSessionIndex].summary" class="text-xs text-slate-600 dark:text-slate-400 italic">
            {{ currentLesson.chapters[activeSessionIndex].summary }}
          </div>

          <div class="text-xs leading-relaxed text-slate-800 dark:text-slate-200 pt-1">
            <MarkdownRenderer
              :content="currentLesson.chapters[activeSessionIndex].content || currentLesson.chapters[activeSessionIndex].content_markdown || 'Under-the-hood technical breakdown in progress.'"
            />
          </div>
        </div>
      </div>

      <div v-if="currentLesson?.core_concepts?.length" class="space-y-2 pt-4 border-t border-slate-100 dark:border-slate-800">
        <h3 class="text-xs font-mono font-bold uppercase tracking-wider text-slate-500">Core Concepts & Invariants</h3>
        <ul class="space-y-1 list-disc pl-5 text-xs text-slate-700 dark:text-slate-300">
          <li v-for="(cc, idx) in currentLesson.core_concepts" :key="idx">
            {{ cc }}
          </li>
        </ul>
      </div>
    </article>
  </div>
</template>
