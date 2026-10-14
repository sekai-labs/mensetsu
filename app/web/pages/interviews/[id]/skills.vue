<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { getSkillNodeLabel, getSkillNodeStatusCategory } from '~/features/interviews/utils/interview.util'
import type { KnowledgeSkillNode } from '~/features/interviews/models/interview.model'
definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceKnowledge } = useInterviewService()

const { data: graphData } = await useAsyncData(
  `skills-map-${workspaceId.value}`,
  () => getWorkspaceKnowledge(workspaceId.value)
)

const activeCategory = ref<string>('all')
const statusFilter = ref<'all' | 'matched' | 'in_progress' | 'gap'>('all')
const searchQuery = ref<string>('')

const categories = computed(() => {
  const set = new Set<string>()
  for (const n of graphData.value?.nodes || []) {
    if (n.category) set.add(n.category)
  }
  return ['all', ...Array.from(set)]
})

const stats = computed(() => {
  const nodes = graphData.value?.nodes || []
  let matched = 0
  let inProgress = 0
  let gap = 0
  for (const n of nodes) {
    const cat = getSkillNodeStatusCategory(n)
    if (cat === 'matched') matched++
    else if (cat === 'in_progress') inProgress++
    else gap++
  }
  return { total: nodes.length, matched, inProgress, gap }
})

const filteredNodes = computed(() => {
  const list = graphData.value?.nodes || []
  return list.filter((n) => {
    const label = getSkillNodeLabel(n).toLowerCase()
    const matchesSearch = !searchQuery.value.trim() || label.includes(searchQuery.value.trim().toLowerCase())
    const matchesCat = activeCategory.value === 'all' || n.category === activeCategory.value
    const statusCat = getSkillNodeStatusCategory(n)
    const matchesStatus = statusFilter.value === 'all' || statusCat === statusFilter.value
    return matchesSearch && matchesCat && matchesStatus
  })
})
</script>

<template>
  <div class="space-y-6">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>COMPETENCY TAXONOMY</span>
          <span>•</span>
          <span>SKILL INVENTORY</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          Skill Map & Category Readiness
        </h1>
      </div>


      <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
        <div class="relative w-full sm:w-56">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search skills..."
            class="w-full px-3 py-1.5 text-xs rounded-md bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>

        <div class="flex items-center gap-1.5 text-xs">
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-mono font-medium transition-colors"
            :class="statusFilter === 'all' ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900' : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'"
            @click="statusFilter = 'all'"
          >
            All ({{ stats.total }})
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-mono font-medium transition-colors"
            :class="statusFilter === 'matched' ? 'bg-emerald-600 text-white' : 'text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40'"
            @click="statusFilter = 'matched'"
          >
            Matched ({{ stats.matched }})
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-mono font-medium transition-colors"
            :class="statusFilter === 'in_progress' ? 'bg-sky-600 text-white' : 'text-sky-600 dark:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-950/40'"
            @click="statusFilter = 'in_progress'"
          >
            In Progress ({{ stats.inProgress }})
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-mono font-medium transition-colors"
            :class="statusFilter === 'gap' ? 'bg-amber-600 text-white' : 'text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40'"
            @click="statusFilter = 'gap'"
          >
            Gap ({{ stats.gap }})
          </button>
        </div>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-1.5 text-xs pb-1">
      <button
        v-for="cat in categories"
        :key="cat"
        type="button"
        class="px-2.5 py-1 rounded-md capitalize font-medium transition-colors"
        :class="activeCategory === cat
          ? 'bg-indigo-600 text-white'
          : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
        @click="activeCategory = cat"
      >
        {{ cat.replace(/_/g, ' ') }}
      </button>
    </div>


    <div v-if="!filteredNodes.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No skills extracted matching selected category.
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
      <div
        v-for="node in filteredNodes"
        :key="node.id || node.slug || node.name"
        class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm space-y-2 hover:border-indigo-500 transition-colors"
      >
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs font-semibold text-slate-900 dark:text-white font-mono truncate" :title="getSkillNodeLabel(node)">
            {{ getSkillNodeLabel(node) }}
          </span>
          <span
            class="text-[10px] font-mono px-1.5 py-0.5 rounded font-bold uppercase shrink-0"
            :class="{
              'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800': getSkillNodeStatusCategory(node) === 'matched',
              'bg-sky-50 text-sky-700 dark:bg-sky-950 dark:text-sky-300 border border-sky-200 dark:border-sky-800': getSkillNodeStatusCategory(node) === 'in_progress',
              'bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300 border border-amber-200 dark:border-amber-800': getSkillNodeStatusCategory(node) === 'gap',
            }"
          >
            {{ getSkillNodeStatusCategory(node) === 'matched' ? 'MATCHED' : (getSkillNodeStatusCategory(node) === 'in_progress' ? 'IN PROGRESS' : 'GAP') }}
          </span>
        </div>

        <div class="flex items-center justify-between text-[11px] text-slate-500 pt-1 border-t border-slate-100 dark:border-slate-800">
          <span class="capitalize">{{ (node.category || 'general').replace(/_/g, ' ') }}</span>
          <span class="capitalize font-mono text-[10px] text-slate-400">{{ node.importance || 'standard' }} importance</span>
        </div>
      </div>
    </div>
  </div>
</template>
