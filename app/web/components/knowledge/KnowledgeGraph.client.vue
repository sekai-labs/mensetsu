<script setup lang="ts">
import { computed, markRaw, ref } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import dagre from '@dagrejs/dagre'
import { ZoomIn, ZoomOut, Maximize2 } from 'lucide-vue-next'
import {
  getSkillNodeLabel,
  getSkillNodeStatusCategory,
} from '~/features/interviews/utils/interview.util'
import KnowledgeNode from '~/components/knowledge/KnowledgeNode.vue'
import type {
  KnowledgeGraphResponse,
  KnowledgeSkillNode,
} from '~/features/interviews/models/interview.model'

const props = defineProps<{
  graphData: KnowledgeGraphResponse | null
  workspaceId: string
}>()

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
  if (!props.graphData || !props.graphData.nodes?.length) {
    return { nodes: [], edges: [] }
  }

  const rawNodes = props.graphData.nodes
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

  const rawEdges = props.graphData.edges || []
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
  <div class="relative h-full w-full">
    <VueFlow
      :nodes="flowElements.nodes"
      :edges="flowElements.edges"
      :node-types="nodeTypes"
      :default-viewport="{ zoom: 0.9, x: 50, y: 50 }"
      :min-zoom="0.2"
      :max-zoom="2"
      fit-view-on-init
      class="h-full w-full"
      @node-click="onNodeClick"
    />

    <div class="absolute top-4 right-4 flex items-center gap-1 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-1 shadow-sm">
      <button
        type="button"
        class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 transition-colors"
        :title="$t('knowledge.zoomIn')"
        :aria-label="$t('knowledge.zoomIn')"
        @click="zoomIn"
      >
        <ZoomIn class="w-4 h-4" />
      </button>
      <button
        type="button"
        class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 transition-colors"
        :title="$t('knowledge.zoomOut')"
        :aria-label="$t('knowledge.zoomOut')"
        @click="zoomOut"
      >
        <ZoomOut class="w-4 h-4" />
      </button>
      <button
        type="button"
        class="p-1.5 rounded text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 transition-colors"
        :title="$t('knowledge.fitView')"
        :aria-label="$t('knowledge.fitView')"
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
          class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 ml-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 rounded"
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
        <NuxtLink :to="`/interviews/${workspaceId}/learning`" class="text-indigo-600 dark:text-indigo-400 hover:underline inline-flex items-center gap-1 font-sans text-xs font-medium focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 rounded">
          <span>View Learning Units for this Skill →</span>
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
