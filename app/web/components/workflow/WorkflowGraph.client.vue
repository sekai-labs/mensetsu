<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { VueFlow, useVueFlow, EdgeLabelRenderer, getSmoothStepPath } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import TaskNode from '~/components/workflow/TaskNode.vue'
import DelegationNode from '~/components/workflow/DelegationNode.vue'

const props = defineProps<{
  nodes: any[]
  edges: any[]
  selectedTaskId: string | null
  selectedAgentId: string | null
}>()

const emit = defineEmits<{
  (e: 'node-click', event: any): void
  (e: 'edge-click', event: { edge: any }): void
}>()

const flowId = 'workflow-run-graph'
const { fitView, zoomIn, zoomOut } = useVueFlow(flowId)

const fitGraph = () => {
  if (!props.nodes.length) return
  nextTick(() => fitView({ padding: 0.2 }))
}

watch(() => props.nodes.length, fitGraph)

defineExpose({ zoomIn, zoomOut, fitView })
</script>

<template>
  <VueFlow
    :id="flowId"
    :nodes="nodes"
    :edges="edges"
    :fit-view-on-init="true"
    :min-zoom="0.2"
    class="h-full w-full"
    @nodes-initialized="fitGraph"
    @node-click="(event) => emit('node-click', event)"
    @edge-click="(event) => emit('edge-click', event)"
  >
    <template #node-task="nodeProps">
      <TaskNode
        :id="nodeProps.id"
        :data="nodeProps.data"
        :selected="selectedTaskId === nodeProps.id"
      />
    </template>

    <template #node-delegation="nodeProps">
      <DelegationNode
        :id="nodeProps.id"
        :label="String(nodeProps.label || '')"
        :data="nodeProps.data"
        :selected="selectedAgentId === nodeProps.id"
      />
    </template>

    <template #edge-smoothstep="edgeProps">
      <g class="cursor-pointer" @click="emit('edge-click', { edge: edgeProps })">
        <path
          :id="edgeProps.id"
          class="vue-flow__edge-path stroke-indigo-400 dark:stroke-indigo-400 stroke-2"
          :d="getSmoothStepPath(edgeProps)[0]"
          :style="edgeProps.style"
        />
        <path
          class="vue-flow__edge-interaction"
          :d="getSmoothStepPath(edgeProps)[0]"
          stroke-opacity="0"
          stroke-width="24"
          fill="none"
        />
        <EdgeLabelRenderer>
          <div
            :style="{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${getSmoothStepPath(edgeProps)[1]}px,${getSmoothStepPath(edgeProps)[2]}px)`,
              pointerEvents: 'all',
            }"
            data-testid="delegation-edge-label"
            class="px-2 py-0.5 rounded border border-indigo-200 dark:border-indigo-800 bg-white dark:bg-slate-900 shadow-xs text-[11px] font-mono text-indigo-600 dark:text-indigo-400 font-semibold cursor-pointer select-none hover:bg-indigo-50 dark:hover:bg-slate-800 transition-colors"
            @click.stop="emit('edge-click', { edge: edgeProps })"
          >
            {{ edgeProps.label || 'Delegation' }}
          </div>
        </EdgeLabelRenderer>
      </g>
    </template>
  </VueFlow>
</template>
