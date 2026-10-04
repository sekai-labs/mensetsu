<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { LearningUnitResponse } from '../models/interview.model'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const lessonId = computed(() => String(route.params.lessonId))

const { getWorkspaceLearning, toggleLearningUnitCompleted } = useInterviewService()
const { data: learningUnits, refresh: refreshLearning } = await useAsyncData(
  `learning-plan-${workspaceId.value}`,
  () => getWorkspaceLearning(workspaceId.value)
)

const currentLesson = computed<LearningUnitResponse | null>(() => {
  const units = learningUnits.value || []
  return units.find((u) => u.id === lessonId.value) || units[0] || null
})

const nextLesson = computed<LearningUnitResponse | null>(() => {
  const units = learningUnits.value || []
  if (!currentLesson.value) return null
  const idx = units.findIndex((u) => u.id === currentLesson.value?.id)
  return idx >= 0 && idx < units.length - 1 ? units[idx + 1] : null
})

const prevLesson = computed<LearningUnitResponse | null>(() => {
  const units = learningUnits.value || []
  if (!currentLesson.value) return null
  const idx = units.findIndex((u) => u.id === currentLesson.value?.id)
  return idx > 0 ? units[idx - 1] : null
})

const activeTab = ref<'lecture' | 'code' | 'diagram' | 'quiz'>('lecture')
const isCompleted = computed(() => currentLesson.value?.completed ?? false)
const toggling = ref<boolean>(false)

const handleToggleCompleted = async () => {
  if (!currentLesson.value) return
  toggling.value = true
  try {
    await toggleLearningUnitCompleted(workspaceId.value, currentLesson.value.id)
    await refreshLearning()
  } catch (e) {
    console.error('Failed to toggle completion', e)
  } finally {
    toggling.value = false
  }
}

const embedUrl = computed(() => {
  const url = currentLesson.value?.video_url
  if (!url) return null
  if (url.includes('youtube.com/watch?v=')) {
    const id = url.split('v=')[1]?.split('&')[0]
    return `https://www.youtube-nocookie.com/embed/${id}`
  }
  if (url.includes('youtu.be/')) {
    const id = url.split('youtu.be/')[1]?.split('?')[0]
    return `https://www.youtube-nocookie.com/embed/${id}`
  }
  return url
})

const activeChapterIndex = ref<number>(0)
const quizRevealed = ref<boolean>(false)
</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-slate-200 dark:border-slate-800 gap-4">
      <div class="flex items-center gap-3">
        <NuxtLink
          :to="`/interviews/${workspaceId}/learning`"
          class="text-xs font-semibold text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 flex items-center gap-1"
        >
          ← Course Syllabus
        </NuxtLink>
        <span class="text-slate-300 dark:text-slate-700">/</span>
        <span class="text-xs font-bold text-slate-900 dark:text-white truncate max-w-sm">
          {{ currentLesson?.title }}
        </span>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="px-3 py-1.5 text-xs font-semibold rounded transition flex items-center gap-1.5 shadow-xs"
          :class="isCompleted
            ? 'bg-emerald-600 text-white hover:bg-emerald-500'
            : 'bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:bg-slate-50'"
          :disabled="toggling"
          @click="handleToggleCompleted"
        >
          <span>{{ isCompleted ? '✓ Completed' : 'Mark as Completed' }}</span>
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
      <div class="lg:col-span-2 space-y-6">
        <div v-if="embedUrl" class="rounded-xl overflow-hidden border border-slate-200 dark:border-slate-800 bg-black aspect-video relative shadow-md">
          <iframe
            :src="embedUrl"
            class="w-full h-full border-0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
            title="Video Lecture"
          ></iframe>
        </div>

        <div
          v-if="currentLesson?.chapters && currentLesson.chapters.length > 0"
          class="p-4 rounded-lg bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-800 space-y-2"
        >
          <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400">Lecture Chapter Marks</div>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="(chap, cidx) in currentLesson.chapters"
              :key="cidx"
              type="button"
              class="px-2.5 py-1 text-xs rounded border transition flex items-center gap-1.5"
              :class="activeChapterIndex === cidx
                ? 'bg-indigo-600 text-white font-semibold border-indigo-600'
                : 'bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:bg-slate-100'"
              @click="activeChapterIndex = cidx"
            >
              <span class="font-mono text-[10px] opacity-80">{{ chap.timestamp }}</span>
              <span>{{ chap.title }}</span>
            </button>
          </div>
        </div>

        <div class="border-b border-slate-200 dark:border-slate-800 flex items-center gap-4 text-xs font-semibold">
          <button
            type="button"
            class="pb-2.5 transition border-b-2"
            :class="activeTab === 'lecture'
              ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
              : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
            @click="activeTab = 'lecture'"
          >
            Lecture Notes & Invariants
          </button>
          <button
            v-if="currentLesson?.diagram"
            type="button"
            class="pb-2.5 transition border-b-2"
            :class="activeTab === 'diagram'
              ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
              : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
            @click="activeTab = 'diagram'"
          >
            System Topology Flow
          </button>
          <button
            type="button"
            class="pb-2.5 transition border-b-2"
            :class="activeTab === 'quiz'
              ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
              : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'"
            @click="activeTab = 'quiz'"
          >
            Knowledge Check Quiz
          </button>
        </div>

        <div v-if="activeTab === 'lecture'" class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
          <div class="prose dark:prose-invert max-w-none text-xs leading-relaxed">
            <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white m-0">
              {{ currentLesson?.title }}
            </h1>
            <p class="text-slate-500 dark:text-slate-400 text-xs">
              {{ currentLesson?.summary }}
            </p>

            <div class="mt-4 p-4 rounded bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 whitespace-pre-line font-sans text-xs text-slate-700 dark:text-slate-300">
              {{ currentLesson?.content_markdown }}
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'diagram' && currentLesson?.diagram" class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
          <div>
            <h3 class="text-sm font-bold text-slate-900 dark:text-white">{{ currentLesson.diagram.caption }}</h3>
            <span class="text-[10px] font-mono text-slate-400 uppercase">{{ currentLesson.diagram.type }}</span>
          </div>

          <div class="p-6 rounded-lg bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700 space-y-4">
            <div class="text-xs font-bold text-slate-400 uppercase tracking-wider">Topology Sequence Nodes</div>
            <div class="flex flex-wrap items-center gap-3">
              <template v-for="(node, nidx) in (Array.isArray(currentLesson.diagram.data?.nodes) ? (currentLesson.diagram.data.nodes as unknown[]) : [])" :key="nidx">
                <div class="px-3 py-2 rounded-md bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-xs text-xs font-mono font-semibold text-indigo-700 dark:text-indigo-300">
                  {{ String(node) }}
                </div>
                <span v-if="Array.isArray(currentLesson.diagram.data?.nodes) && nidx < currentLesson.diagram.data.nodes.length - 1" class="text-slate-400 font-bold">
                  →
                </span>
              </template>
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'quiz'" class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
          <div class="space-y-1">
            <h3 class="text-sm font-bold text-slate-900 dark:text-white">Knowledge Check & Interview Validation</h3>
            <p class="text-xs text-slate-500">Test your mastery of this concept before advancing to dependent units.</p>
          </div>

          <div class="p-4 rounded-md border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-850 space-y-3">
            <div class="text-xs font-semibold text-slate-800 dark:text-slate-200">
              Q: In high-scale distributed systems, what is the primary invariant required to prevent duplicate execution when retrying network timeouts?
            </div>

            <div class="space-y-2">
              <label class="flex items-center gap-2 p-2.5 rounded bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-xs cursor-pointer hover:border-indigo-500">
                <input type="radio" name="quiz" class="text-indigo-600" />
                <span>Deterministic Client Idempotency Keys stored with transaction state</span>
              </label>
              <label class="flex items-center gap-2 p-2.5 rounded bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-xs cursor-pointer hover:border-indigo-500">
                <input type="radio" name="quiz" class="text-indigo-600" />
                <span>Increasing HTTP timeout to 60 seconds</span>
              </label>
            </div>

            <button
              type="button"
              class="px-3 py-1.5 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-500 transition"
              @click="quizRevealed = !quizRevealed"
            >
              {{ quizRevealed ? 'Hide Explanation' : 'Check Answer & Interview Rubric' }}
            </button>

            <div v-if="quizRevealed" class="p-3 rounded bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-xs text-emerald-800 dark:text-emerald-300 space-y-1">
              <strong>Staff Engineer Evaluation Tip:</strong>
              <p class="m-0">
                Correct: Idempotency keys must be validated atomically inside the persistence layer (e.g. unique constraint or database transaction). Merely checking in application cache leaves a race condition window open.
              </p>
            </div>
          </div>
        </div>

        <div class="flex items-center justify-between pt-4 border-t border-slate-200 dark:border-slate-800">
          <NuxtLink
            v-if="prevLesson"
            :to="`/interviews/${workspaceId}/learning/${prevLesson.id}`"
            class="px-3 py-1.5 text-xs font-medium rounded border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition"
          >
            ← {{ prevLesson.title }}
          </NuxtLink>
          <div v-else></div>

          <NuxtLink
            v-if="nextLesson"
            :to="`/interviews/${workspaceId}/learning/${nextLesson.id}`"
            class="px-4 py-1.5 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-500 transition"
          >
            Next: {{ nextLesson.title }} →
          </NuxtLink>
        </div>
      </div>

      <aside class="sticky top-20 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-4 shadow-sm space-y-3">
        <div class="flex items-center justify-between pb-2 border-b border-slate-100 dark:border-slate-800">
          <span class="text-xs font-bold uppercase tracking-wider text-slate-400">Course Syllabus</span>
          <span class="text-xs font-mono text-slate-500">{{ learningUnits?.length || 0 }} Units</span>
        </div>

        <nav class="space-y-1">
          <NuxtLink
            v-for="(unit, idx) in (learningUnits || [])"
            :key="unit.id"
            :to="`/interviews/${workspaceId}/learning/${unit.id}`"
            class="p-2 rounded text-xs transition flex items-start gap-2.5 block"
            :class="unit.id === currentLesson?.id
              ? 'bg-indigo-50 dark:bg-indigo-950/70 text-indigo-700 dark:text-indigo-300 font-semibold'
              : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800'"
          >
            <span
              class="w-5 h-5 rounded-full text-[10px] font-mono font-bold flex items-center justify-center shrink-0 mt-0.5"
              :class="unit.completed ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300' : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300'"
            >
              {{ unit.completed ? '✓' : idx + 1 }}
            </span>
            <div class="min-w-0 flex-1">
              <div class="truncate text-xs">{{ unit.title }}</div>
              <div class="text-[10px] text-slate-400 font-mono">{{ unit.estimated_minutes }}m • {{ unit.skill }}</div>
            </div>
          </NuxtLink>
        </nav>
      </aside>
    </div>
  </div>
</template>
