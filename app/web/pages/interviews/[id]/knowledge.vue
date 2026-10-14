<script setup lang="ts">
import { useRoute } from 'vue-router'
import { VueFlow } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { KnowledgeSkillNode } from '~/features/interviews/models/interview.model'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceKnowledge } = useInterviewService()

const { data: graphData } = await useAsyncData(
  `knowledge-graph-${workspaceId.value}`,
  () => getWorkspaceKnowledge(workspaceId.value)
)

const selectedNode = ref<KnowledgeSkillNode | null>(null)


const flowElements = computed(() => {
  if (!graphData.value) return []

  const nodes = graphData.value.nodes.map((n, idx) => {

    const col = idx % 3
    const row = Math.floor(idx / 3)
    return {
      id: n.id,
      label: n.label,
      position: { x: col * 260 + 50, y: row * 140 + 50 },
      data: n,
      class: n.is_matched
        ? 'px-3 py-2 rounded-md border text-xs font-mono font-medium shadow-sm bg-emerald-50 dark:bg-emerald-950/80 border-emerald-300 dark:border-emerald-700 text-emerald-900 dark:text-emerald-100 cursor-pointer'
        : 'px-3 py-2 rounded-md border text-xs font-mono font-medium shadow-sm bg-amber-50 dark:bg-amber-950/80 border-amber-300 dark:border-amber-700 text-amber-900 dark:text-amber-100 cursor-pointer',
    }
  })

  const edges = graphData.value.edges.map((e, idx) => {
    return {
      id: `edge-${idx}`,
      source: e.source,
      target: e.target,
      label: e.relationship,
      animated: true,
      style: { stroke: '#6366f1', strokeWidth: 1.5 },
    }
  })

  return [...nodes, ...edges]
})

const onNodeClick = (event: { node: { data: KnowledgeSkillNode } }) => {
  selectedNode.value = event.node.data
}
</script>

<template>
  <div class="space-y-4 flex flex-col h-[calc(100vh-140px)] min-h-[550px]">

    <div class="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-slate-800 shrink-0">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interactive Knowledge Graph
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          Topological skill dependencies and architecture concepts.
        </p>
      </div>

      <div class="flex items-center gap-3 text-xs">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-slate-600 dark:text-slate-400">Matched Candidate Skill</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-amber-500"></span>
          <span class="text-slate-600 dark:text-slate-400">Target Role Skill Gap</span>
        </div>
      </div>
    </div>


    <div class="flex-1 w-full relative border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-slate-900/5 dark:bg-slate-950">
      <VueFlow
        :elements="flowElements"
        :default-viewport="{ zoom: 1, x: 0, y: 0 }"
        :min-zoom="0.2"
        :max-zoom="2"
        fit-view-on-init
        class="w-full h-full"
        @node-click="onNodeClick"
      />


      <div
        v-if="selectedNode"
        class="absolute bottom-4 right-4 w-72 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-4 shadow-lg space-y-2 text-xs"
      >
        <div class="flex items-center justify-between">
          <span class="font-mono font-bold text-slate-900 dark:text-white">{{ selectedNode.label }}</span>
          <button
            type="button"
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
            @click="selectedNode = null"
          >
            ✕
          </button>
        </div>
        <div class="text-[11px] text-slate-500 space-y-1">
          <div><span class="font-medium">Category:</span> {{ selectedNode.category }}</div>
          <div><span class="font-medium">Importance:</span> {{ selectedNode.importance }}</div>
          <div>
            <span class="font-medium">Status:</span>
            <span :class="selectedNode.is_matched ? 'text-emerald-600' : 'text-amber-600'">
              {{ selectedNode.is_matched ? 'Verified Candidate Strength' : 'Role Gap to Prepare' }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
