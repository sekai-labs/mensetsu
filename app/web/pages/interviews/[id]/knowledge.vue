<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'

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

const graphNodeCount = computed(() => graphData.value?.nodes?.length ?? 0)
const graphEdgeCount = computed(() => graphData.value?.edges?.length ?? 0)
</script>

<template>
  <div class="space-y-4 flex flex-col h-[calc(100vh-140px)] min-h-[550px]">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-200 dark:border-slate-800 shrink-0 gap-3">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('knowledge.title') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          {{ $t('knowledge.subtitle') }}
        </p>
      </div>

      <div class="flex items-center gap-4 text-xs font-mono">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-slate-600 dark:text-slate-400">{{ $t('knowledge.matchedCompetency') }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-sky-500"></span>
          <span class="text-slate-600 dark:text-slate-400">{{ $t('knowledge.inProgress') }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-amber-500"></span>
          <span class="text-slate-600 dark:text-slate-400">{{ $t('knowledge.skillGap') }}</span>
        </div>
      </div>
    </div>

    <div
      v-if="graphNodeCount === 0"
      class="flex-1 flex items-center justify-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400 font-mono"
    >
      {{ $t('knowledge.empty') }}
    </div>

    <div
      v-else
      class="flex-1 w-full relative border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-slate-900/5 dark:bg-slate-950"
    >
      <ClientOnly>
        <KnowledgeGraph :graph-data="graphData" :workspace-id="workspaceId" />
        <template #fallback>
          <div class="absolute inset-0 flex items-center justify-center p-6" role="status" aria-live="polite">
            <div class="w-full max-w-sm rounded-lg border border-slate-200 bg-white p-5 text-center shadow-sm dark:border-slate-800 dark:bg-slate-900">
              <div class="mx-auto grid w-40 grid-cols-3 gap-1.5" aria-hidden="true">
                <span class="h-3 rounded bg-emerald-200 dark:bg-emerald-900"></span>
                <span class="h-3 rounded bg-sky-200 dark:bg-sky-900"></span>
                <span class="h-3 rounded bg-amber-200 dark:bg-amber-900"></span>
                <span class="h-3 rounded bg-slate-200 dark:bg-slate-800"></span>
                <span class="h-3 rounded bg-indigo-200 dark:bg-indigo-900"></span>
                <span class="h-3 rounded bg-slate-200 dark:bg-slate-800"></span>
              </div>
              <p class="mt-3 text-xs font-semibold text-slate-900 dark:text-white">Knowledge graph</p>
              <p class="mt-1 text-[11px] font-mono text-slate-500 dark:text-slate-400">
                {{ graphNodeCount }} competencies · {{ graphEdgeCount }} prerequisite links
              </p>
            </div>
          </div>
        </template>
      </ClientOnly>
    </div>
  </div>
</template>
