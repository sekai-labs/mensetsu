<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { KnowledgeSkillNode } from '../models/interview.model'
import { VueFlow } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'

interface NodeClickPayload {
  node?: {
    data?: KnowledgeSkillNode
  }
}

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceKnowledge } = useInterviewService()
const { data: knowledgeData, pending } = await useAsyncData(
  `knowledge-graph-${workspaceId.value}`,
  () => getWorkspaceKnowledge(workspaceId.value)
)

const searchQuery = ref('')
const selectedCategory = ref('all')
const selectedNode = ref<KnowledgeSkillNode | null>(null)

const categories = computed(() => {
  const nodes = knowledgeData.value?.nodes || []
  const cats = new Set(nodes.map((n) => n.category))
  return ['all', ...Array.from(cats)]
})

const filteredNodes = computed(() => {
  if (!knowledgeData.value?.nodes) return []
  return knowledgeData.value.nodes.filter((n) => {
    const matchesSearch = n.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesCategory =
      selectedCategory.value === 'all' || n.category === selectedCategory.value
    return matchesSearch && matchesCategory
  })
})

const flowNodes = computed(() => {
  return filteredNodes.value.map((n, i) => ({
    id: n.id,
    label: n.name,
    position: { x: (i % 4) * 240 + 40, y: Math.floor(i / 4) * 120 + 40 },
    data: n,
    class: `px-3 py-2 rounded-lg border text-xs font-medium shadow-sm transition-all cursor-pointer ${
      n.candidate_status === 'mastered'
        ? 'bg-emerald-50 dark:bg-emerald-950/70 border-emerald-300 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300'
        : n.candidate_status === 'gap'
        ? 'bg-amber-50 dark:bg-amber-950/70 border-amber-300 dark:border-amber-800 text-amber-800 dark:text-amber-300'
        : 'bg-indigo-50 dark:bg-indigo-950/70 border-indigo-300 dark:border-indigo-800 text-indigo-800 dark:text-indigo-300'
    }`,
  }))
})

const flowEdges = computed(() => {
  if (!knowledgeData.value?.edges) return []
  const visibleNodeIds = new Set(filteredNodes.value.map((n) => n.id))
  return knowledgeData.value.edges
    .filter((e) => visibleNodeIds.has(e.source) && visibleNodeIds.has(e.target))
    .map((e, idx) => ({
      id: `edge-${idx}`,
      source: e.source,
      target: e.target,
      label: e.relation,
      animated: true,
    }))
})

const onNodeClick = (event: NodeClickPayload) => {
  if (event?.node?.data) {
    selectedNode.value = event.node.data
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interactive Knowledge Graph
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Explore technical prerequisite dependencies, role competencies, and verified company stack nodes.
        </p>
      </div>

      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 w-full md:w-auto">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter nodes..."
          class="px-3 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-full sm:w-48"
        />
        <select
          v-model="selectedCategory"
          class="px-2.5 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-700 dark:text-slate-300 focus:outline-none focus:ring-1 focus:ring-indigo-500 w-full sm:w-auto"
        >
          <option v-for="cat in categories" :key="cat" :value="cat">
            {{ cat === 'all' ? 'All Categories' : cat }}
          </option>
        </select>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
      <div class="lg:col-span-3 h-[420px] sm:h-[500px] lg:h-[600px] rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 relative overflow-hidden shadow-sm">
        <div v-if="pending" class="absolute inset-0 flex items-center justify-center bg-white/70 dark:bg-slate-900/70 z-10 text-xs text-slate-500">
          Loading graph nodes...
        </div>

        <div v-else-if="!knowledgeData?.nodes || knowledgeData.nodes.length === 0" class="absolute inset-0 flex items-center justify-center text-xs text-slate-500">
          Knowledge graph will populate once research activities complete.
        </div>

        <VueFlow
          v-else
          :nodes="flowNodes"
          :edges="flowEdges"
          :default-zoom="1.1"
          :fit-view-on-init="true"
          @node-click="onNodeClick"
        />
      </div>

      <div class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-500 border-b border-slate-100 dark:border-slate-800 pb-2">
          Node Inspector
        </h2>

        <div v-if="selectedNode" class="space-y-3">
          <div>
            <span class="text-[10px] font-mono uppercase tracking-wider text-slate-400">{{ selectedNode.category }}</span>
            <h3 class="text-base font-bold text-slate-900 dark:text-white">{{ selectedNode.name }}</h3>
          </div>

          <div class="text-xs space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-slate-500">Status:</span>
              <span class="font-semibold capitalize text-slate-900 dark:text-white">
                {{ selectedNode.candidate_status.replace('_', ' ') }}
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-500">Importance:</span>
              <span class="font-semibold capitalize text-slate-900 dark:text-white">
                {{ selectedNode.importance }}
              </span>
            </div>
          </div>

          <div class="pt-3 border-t border-slate-100 dark:border-slate-800">
            <NuxtLink
              :to="`/interviews/${workspaceId}/learning`"
              class="w-full text-center block px-3 py-1.5 rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 text-xs font-semibold hover:bg-indigo-100"
            >
              Study this topic →
            </NuxtLink>
          </div>
        </div>

        <div v-else class="text-xs text-slate-400 italic py-8 text-center">
          Click on any skill node to view detailed prerequisites and curriculum connections.
        </div>
      </div>
    </div>
  </div>
</template>
