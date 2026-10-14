<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'

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

const categories = computed(() => {
  const set = new Set<string>()
  for (const n of graphData.value?.nodes || []) {
    set.add(n.category)
  }
  return ['all', ...Array.from(set)]
})

const filteredNodes = computed(() => {
  const list = graphData.value?.nodes || []
  if (activeCategory.value === 'all') return list
  return list.filter((n) => n.category === activeCategory.value)
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


      <div class="flex flex-wrap items-center gap-1.5 text-xs">
        <button
          v-for="cat in categories"
          :key="cat"
          type="button"
          class="px-2.5 py-1 rounded-md capitalize font-medium transition-colors"
          :class="activeCategory === cat
            ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
            : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'"
          @click="activeCategory = cat"
        >
          {{ cat.replace(/_/g, ' ') }}
        </button>
      </div>
    </div>


    <div v-if="!filteredNodes.length" class="p-8 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
      No skills extracted matching selected category.
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
      <div
        v-for="node in filteredNodes"
        :key="node.id"
        class="p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-sm space-y-2 hover:border-indigo-500 transition-colors"
      >
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs font-semibold text-slate-900 dark:text-white font-mono truncate">
            {{ node.label }}
          </span>
          <span
            class="text-[10px] font-mono px-1.5 py-0.5 rounded font-bold uppercase shrink-0"
            :class="node.is_matched
              ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800'
              : 'bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300 border border-amber-200 dark:border-amber-800'"
          >
            {{ node.is_matched ? 'MATCHED' : 'GAP' }}
          </span>
        </div>

        <div class="flex items-center justify-between text-[11px] text-slate-500 pt-1 border-t border-slate-100 dark:border-slate-800">
          <span class="capitalize">{{ node.category.replace(/_/g, ' ') }}</span>
          <span class="capitalize font-mono text-[10px] text-slate-400">{{ node.importance }} importance</span>
        </div>
      </div>
    </div>
  </div>
</template>
