<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { KnowledgeSkillNode } from '../models/interview.model'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceKnowledge } = useInterviewService()
const { data: knowledgeData } = await useAsyncData(
  `skills-page-${workspaceId.value}`,
  () => getWorkspaceKnowledge(workspaceId.value)
)

const activeStatusFilter = ref<'all' | 'mastered' | 'in_progress' | 'gap'>('all')

const filteredSkills = computed(() => {
  const nodes = knowledgeData.value?.nodes || []
  if (activeStatusFilter.value === 'all') return nodes
  return nodes.filter((n) => n.candidate_status === activeStatusFilter.value)
})

const selectedSkill = ref<KnowledgeSkillNode | null>(null)
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Role-Specific Skill Map
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Catalog of required concepts categorized by candidate mastery and priority.
        </p>
      </div>

      <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none pb-1 sm:pb-0 w-full sm:w-auto">
        <button
          v-for="st in ['all', 'mastered', 'in_progress', 'gap'] as const"
          :key="st"
          @click="activeStatusFilter = st"
          class="px-2.5 py-1 rounded text-xs font-semibold uppercase tracking-wider transition shrink-0"
          :class="activeStatusFilter === st
            ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
            : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-200'"
        >
          {{ st.replace('_', ' ') }}
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
      <div
        v-for="skill in filteredSkills"
        :key="skill.id"
        @click="selectedSkill = skill"
        class="p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm cursor-pointer hover:border-indigo-400 transition space-y-3"
      >
        <div class="flex items-center justify-between">
          <span class="text-[10px] font-mono uppercase tracking-wider text-slate-400">{{ skill.category }}</span>
          <span
            class="text-[10px] px-2 py-0.5 rounded font-semibold uppercase tracking-wider"
            :class="{
              'bg-emerald-50 text-emerald-700 border border-emerald-200': skill.candidate_status === 'mastered',
              'bg-indigo-50 text-indigo-700 border border-indigo-200': skill.candidate_status === 'in_progress',
              'bg-amber-50 text-amber-700 border border-amber-200': skill.candidate_status === 'gap',
            }"
          >
            {{ skill.candidate_status.replace('_', ' ') }}
          </span>
        </div>

        <h3 class="text-sm font-semibold text-slate-900 dark:text-white">{{ skill.name }}</h3>

        <div class="flex items-center justify-between text-xs text-slate-500 pt-2 border-t border-slate-100 dark:border-slate-800">
          <span class="capitalize">Priority: {{ skill.importance }}</span>
          <span class="text-indigo-600 text-xs">Inspect →</span>
        </div>
      </div>
    </div>

    <div v-if="selectedSkill" class="p-6 rounded-lg border border-indigo-200 dark:border-indigo-900/60 bg-indigo-50/30 dark:bg-indigo-950/20 shadow-sm space-y-3">
      <div class="flex items-center justify-between">
        <h3 class="text-base font-bold text-slate-900 dark:text-white">{{ selectedSkill.name }}</h3>
        <button @click="selectedSkill = null" class="text-xs text-slate-400 hover:text-slate-600">Close ✕</button>
      </div>
      <p class="text-xs text-slate-600 dark:text-slate-300">
        Category: <strong>{{ selectedSkill.category }}</strong> | Status: <strong class="capitalize">{{ selectedSkill.candidate_status.replace('_', ' ') }}</strong>
      </p>
      <div class="flex items-center gap-3 pt-2">
        <NuxtLink :to="`/interviews/${workspaceId}/knowledge`" class="text-xs font-semibold text-indigo-600 hover:underline">
          View in Knowledge Graph →
        </NuxtLink>
        <NuxtLink :to="`/interviews/${workspaceId}/learning`" class="text-xs font-semibold text-indigo-600 hover:underline">
          View Curriculum Lessons →
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
