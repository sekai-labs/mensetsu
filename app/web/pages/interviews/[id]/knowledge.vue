<script setup lang="ts">
import { computed, ref, markRaw } from 'vue'
import { useRoute } from 'vue-router'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import dagre from '@dagrejs/dagre'
import { ZoomIn, ZoomOut, Maximize2 } from 'lucide-vue-next'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { getSkillNodeLabel, getSkillNodeStatusCategory } from '~/features/interviews/utils/interview.util'
import KnowledgeNode from '~/components/knowledge/KnowledgeNode.vue'
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
const nodeTypes = {
  knowledge: markRaw(KnowledgeNode),
}

const { fitView, zoomIn, zoomOut } = useVueFlow()

const layoutDagre = (
  nodes: Array<{ id: string; type: string; data: KnowledgeSkillNode; position: { x: number; y: number } }>,
  edges: Array<{ id: string; source: string; target: string; label?: string }>,
  direction = 'TB',
  nodeWidth = 240,
  nodeHeight = 90,
  nodesep = 50,
  ranksep = 80
) => {
  const dagreGraph = new dagre.graphlib.Graph()
  dagreGraph.setDefaultEdgeLabel(() => ({}))
  dagreGraph.setGraph({ rankdir: direction, nodesep, ranksep })

  for (const node of nodes) {
    dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight })
  }

  for (const edge of edges) {
    dagreGraph.setEdge(edge.source, edge.target)
  }

  dagre.layout(dagreGraph)

  const halfW = nodeWidth / 2
  const halfH = nodeHeight / 2

  return nodes.map((node) => {
    const pos = dagreGraph.node(node.id)
    return {
      ...node,
      position: {
        x: pos ? pos.x - halfW : 0,
        y: pos ? pos.y - halfH : 0,
      },
    }
  })
}

const flowElements = computed(() => {
  if (!graphData.value || !graphData.value.nodes?.length) {
    return { nodes: [], edges: [] }
  }

  const rawNodes = graphData.value.nodes
  // Create mapping table for matching edges whether by id, slug, or name (lowercase normalized)
  const nodeKeyMap = new Map<string, string>()
  for (const n of rawNodes) {
    const canonicalId = n.id || n.slug || n.name || ''
    if (n.id) nodeKeyMap.set(n.id.toLowerCase(), canonicalId)
    if (n.slug) nodeKeyMap.set(n.slug.toLowerCase(), canonicalId)
    if (n.name) nodeKeyMap.set(n.name.toLowerCase(), canonicalId)
  }

  const nodes = rawNodes.map((n) => {
    const canonicalId = n.id || n.slug || n.name || ''
    return {
      id: canonicalId,
      type: 'knowledge',
      data: n,
      position: { x: 0, y: 0 },
    }
  })

  const rawEdges = graphData.value.edges || []
  const validEdges: Array<{
    id: string
    source: string
    target: string
    label?: string
    animated: boolean
    style: { stroke: string; strokeWidth: number }
  }> = []

  rawEdges.forEach((e, idx) => {
    const srcKey = (e.source || '').toLowerCase()
    const tgtKey = (e.target || '').toLowerCase()
    const resolvedSrc = nodeKeyMap.get(srcKey) || e.source
    const resolvedTgt = nodeKeyMap.get(tgtKey) || e.target

    if (resolvedSrc && resolvedTgt && resolvedSrc !== resolvedTgt) {
      validEdges.push({
        id: `edge-${idx}-${resolvedSrc}-${resolvedTgt}`,
        source: resolvedSrc,
        target: resolvedTgt,
        label: e.relation || e.relationship || 'prerequisite',
        animated: true,
        style: { stroke: '#6366f1', strokeWidth: 1.5 },
      })
    }
  })

  const layoutedNodes = layoutDagre(nodes, validEdges)
  return { nodes: layoutedNodes, edges: validEdges }
})

const onNodeClick = (event: { node: { data: KnowledgeSkillNode } }) => {
  selectedNode.value = event.node.data
}
</script>

<template>
  <div class="space-y-4 flex flex-col h-[calc(100vh-140px)] min-h-[550px]">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-200 dark:border-slate-800 shrink-0 gap-3">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Interactive Knowledge Graph
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          Topological skill dependencies, verified candidate strengths, and targeted gaps.
        </p>
      </div>

      <div class="flex items-center gap-4 text-xs font-mono">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-slate-600 dark:text-slate-400">Matched Competency</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-sky-500"></span>
          <span class="text-slate-600 dark:text-slate-400">In Progress</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-amber-500"></span>
          <span class="text-slate-600 dark:text-slate-400">Skill Gap</span>
        </div>
      </div>
    </div>

    <div
      v-if="!flowElements.nodes.length"
      class="flex-1 flex items-center justify-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400 font-mono"
    >
      Synthesizing knowledge graph relationships...
    </div>

    <div
      v-else
      class="flex-1 w-full relative border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-slate-900/5 dark:bg-slate-950"
    >
      <VueFlow
        :nodes="flowElements.nodes"
        :edges="flowElements.edges"
        :node-types="nodeTypes"
        :default-viewport="{ zoom: 0.9, x: 50, y: 50 }"
        :min-zoom="0.2"
        :max-zoom="2"
        fit-view-on-init
        class="w-full h-full"
        @node-click="onNodeClick"
      />

      <div class="absolute top-4 right-4 flex items-center gap-1 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-1 shadow-sm">
        <button
          type="button"
          class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Zoom In"
          aria-label="Zoom In"
          @click="zoomIn"
        >
          <ZoomIn class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Zoom Out"
          aria-label="Zoom Out"
          @click="zoomOut"
        >
          <ZoomOut class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Fit View"
          aria-label="Fit View"
          @click="fitView"
        >
          <Maximize2 class="w-4 h-4" />
        </button>
      </div>

      <div
        v-if="selectedNode"
        class="absolute bottom-4 right-4 w-80 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-4 shadow-xl space-y-3 text-xs"
      >
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-2">
          <span class="font-mono font-bold text-slate-900 dark:text-white truncate">
            {{ getSkillNodeLabel(selectedNode) }}
          </span>
          <button
            type="button"
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 ml-2"
            aria-label="Close skill details"
            @click="selectedNode = null"
          >
            ✕
          </button>
        </div>

        <div class="space-y-1.5 text-slate-600 dark:text-slate-300 font-mono text-[11px]">
          <div class="flex justify-between">
            <span class="text-slate-400">Category:</span>
            <span class="capitalize text-slate-900 dark:text-white font-medium">{{ (selectedNode.category || 'general').replace(/_/g, ' ') }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-400">Importance:</span>
            <span class="capitalize text-slate-900 dark:text-white font-medium">{{ selectedNode.importance || 'core' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-400">Readiness:</span>
            <span
              class="font-bold uppercase text-[10px] px-1.5 py-0.5 rounded"
              :class="{
                'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300': getSkillNodeStatusCategory(selectedNode) === 'matched',
                'bg-sky-50 text-sky-700 dark:bg-sky-950 dark:text-sky-300': getSkillNodeStatusCategory(selectedNode) === 'in_progress',
                'bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300': getSkillNodeStatusCategory(selectedNode) === 'gap',
              }"
            >
              {{ getSkillNodeStatusCategory(selectedNode) === 'matched' ? 'Verified Strength' : (getSkillNodeStatusCategory(selectedNode) === 'in_progress' ? 'In Progress' : 'Target Skill Gap') }}
            </span>
          </div>
        </div>

        <div class="pt-2 border-t border-slate-100 dark:border-slate-800">
          <NuxtLink :to="`/interviews/${workspaceId}/learning`" class="text-indigo-600 dark:text-indigo-400 hover:underline inline-flex items-center gap-1 font-sans text-xs font-medium">
            <span>View Learning Units for this Skill →</span>
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
