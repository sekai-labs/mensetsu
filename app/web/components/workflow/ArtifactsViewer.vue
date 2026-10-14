<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  FileText,
  HelpCircle,
  ExternalLink,
  ShieldCheck,
  Search,
  Filter,
  Layers,
  ChevronDown,
  ChevronUp,
  CheckCircle,
} from 'lucide-vue-next'
import type { Artifact, ArtifactType } from '~/models/generated/workflows.v1'

const props = defineProps<{
  artifacts: Artifact[]
}>()

const emit = defineEmits<{
  (e: 'select-task', taskId: string): void
  (e: 'select-execution', executionId: string): void
}>()

const filterType = ref<string>('all')
const searchQuery = ref<string>('')
const expandedArtifacts = ref<Record<string, boolean>>({})

const toggleExpand = (id: string) => {
  expandedArtifacts.value[id] = !expandedArtifacts.value[id]
}

const artifactTypes = computed(() => {
  const set = new Set<string>()
  for (const a of props.artifacts) {
    if (a.type) {
      set.add(a.type)
    }
  }
  return Array.from(set).sort()
})

const filteredArtifacts = computed(() => {
  return props.artifacts.filter((a) => {
    if (filterType.value !== 'all' && a.type !== filterType.value) {
      return false
    }

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      const titleMatch = a.title.toLowerCase().includes(q)
      const contentMatch = JSON.stringify(a.content).toLowerCase().includes(q)
      if (!titleMatch && !contentMatch) return false
    }

    return true
  })
})

const getTypeConfig = (type?: ArtifactType) => {
  switch (type) {
    case 'research_report':
      return {
        badge: 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/60 dark:text-indigo-400 dark:border-indigo-800',
        icon: FileText,
        label: 'Research Report',
      }
    case 'finding':
      return {
        badge: 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-400 dark:border-emerald-800',
        icon: CheckCircle,
        label: 'Finding',
      }
    case 'evidence':
      return {
        badge: 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-400 dark:border-emerald-800',
        icon: ShieldCheck,
        label: 'Evidence',
      }
    case 'question_set':
      return {
        badge: 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/60 dark:text-indigo-400 dark:border-indigo-800',
        icon: HelpCircle,
        label: 'Question Set',
      }
    case 'recommendation':
      return {
        badge: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/60 dark:text-amber-400 dark:border-amber-800',
        icon: Layers,
        label: 'Recommendation',
      }
    case 'validation_result':
      return {
        badge: 'bg-slate-50 text-slate-700 border-slate-200 dark:bg-slate-900 dark:text-slate-300 dark:border-slate-800',
        icon: ShieldCheck,
        label: 'Validation Result',
      }
    default:
      return {
        badge: 'bg-slate-50 text-slate-700 border-slate-200 dark:bg-slate-900 dark:text-slate-300 dark:border-slate-800',
        icon: FileText,
        label: type || 'Artifact',
      }
  }
}
</script>

<template>
  <div class="space-y-4">

    <div class="flex flex-wrap items-center justify-between gap-3 p-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg">
      <div class="flex items-center gap-2">
        <Filter class="w-3.5 h-3.5 text-slate-400" />
        <select
          v-model="filterType"
          class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-2.5 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono"
        >
          <option value="all">All Artifact Types ({{ artifacts.length }})</option>
          <option v-for="t in artifactTypes" :key="t" :value="t">{{ t }}</option>
        </select>
      </div>

      <div class="relative w-full sm:w-64">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search artifacts..."
          class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded pl-8 pr-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-mono placeholder:text-slate-400"
        />
        <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-2" />
      </div>
    </div>


    <div v-if="filteredArtifacts.length === 0" class="p-12 text-center text-xs text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg">
      No artifacts matching criteria.
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="art in filteredArtifacts"
        :key="art.id"
        class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden shadow-xs hover:border-slate-300 dark:hover:border-slate-700 transition-colors"
        data-testid="artifact-card"
        :data-artifact-type="art.type"
      >

        <div class="p-3.5 flex items-start justify-between gap-3 bg-slate-50/50 dark:bg-slate-800/30 border-b border-slate-100 dark:border-slate-800">
          <div class="space-y-1 min-w-0">
            <div class="flex items-center gap-2">
              <span
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium border uppercase font-mono"
                :class="getTypeConfig(art.type).badge"
              >
                <component :is="getTypeConfig(art.type).icon" class="w-3 h-3 shrink-0" />
                <span>{{ getTypeConfig(art.type).label }}</span>
              </span>
              <span class="font-mono text-[10px] text-slate-400">{{ art.id }}</span>
            </div>
            <div class="text-sm font-semibold text-slate-900 dark:text-white truncate">
              {{ art.title }}
            </div>
          </div>


          <div class="flex items-center gap-3 shrink-0 text-xs">
            <button
              v-if="art.task_id"
              class="text-indigo-600 dark:text-indigo-400 hover:underline font-mono text-[11px] flex items-center gap-1"
              @click="emit('select-task', art.task_id)"
            >
              <span>Task: {{ art.task_id }}</span>
            </button>
            <button
              v-if="art.agent_execution_id"
              class="text-indigo-600 dark:text-indigo-400 hover:underline font-mono text-[11px] flex items-center gap-1"
              data-testid="artifact-agent-link"
              @click="emit('select-execution', art.agent_execution_id)"
            >
              <span>Agent: {{ art.agent_execution_id }}</span>
            </button>
            <button
              class="p-1 rounded text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
              @click="toggleExpand(art.id)"
            >
              <ChevronUp v-if="expandedArtifacts[art.id]" class="w-4 h-4" />
              <ChevronDown v-else class="w-4 h-4" />
            </button>
          </div>
        </div>


        <div v-if="art.type === 'evidence' && art.evidence" class="p-3 bg-emerald-50/20 dark:bg-emerald-950/10 border-b border-emerald-100 dark:border-emerald-900/40 text-xs space-y-2">
          <div class="flex items-center justify-between text-slate-500 font-mono text-[11px]">
            <a
              :href="art.evidence.source_url"
              target="_blank"
              rel="noopener noreferrer"
              class="text-indigo-600 dark:text-indigo-400 hover:underline flex items-center gap-1 font-semibold truncate max-w-md"
            >
              <span>{{ art.evidence.title || art.evidence.source_url }}</span>
              <ExternalLink class="w-3 h-3 shrink-0" />
            </a>
            <div class="flex items-center gap-3 shrink-0">
              <span v-if="art.evidence.confidence !== undefined">Confidence: {{ Math.round(art.evidence.confidence * 100) }}%</span>
              <span>Retrieved: {{ art.evidence.retrieved_at }}</span>
            </div>
          </div>

          <div class="p-2.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded font-serif italic text-slate-700 dark:text-slate-300 leading-relaxed">
            "{{ art.evidence.snippet }}"
          </div>
        </div>


        <div v-if="expandedArtifacts[art.id]" class="p-3 bg-slate-950 border-t border-slate-800 text-slate-200 font-mono text-xs overflow-x-auto max-h-96">
          <pre class="leading-relaxed whitespace-pre-wrap">{{ JSON.stringify(art.content, null, 2) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>
