<script setup lang="ts">
import { ref, computed } from 'vue'
import { onClickOutside, useEventListener } from '@vueuse/core'
import {
  Activity,
  ChevronDown,
  ExternalLink,
  Wifi,
  WifiOff,
} from 'lucide-vue-next'
import type { WorkspaceStatus, PipelinePhase } from '~/models/common.model'

interface Props {
  status?: WorkspaceStatus | string
  currentPhase?: PipelinePhase | string
  progressPercent?: number
  isConnected?: boolean
  message?: string
  workspaceId?: string
}

const props = withDefaults(defineProps<Props>(), {
  status: 'pending',
  currentPhase: 'preparing_input',
  progressPercent: 0,
  isConnected: false,
  message: '',
  workspaceId: '',
})

const isOpen = ref(false)
const popoverRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)

onClickOutside(popoverRef, () => {
  if (isOpen.value) {
    isOpen.value = false
  }
})

useEventListener('keydown', (e: KeyboardEvent) => {
  if (e.key === 'Escape' && isOpen.value) {
    isOpen.value = false
    triggerRef.value?.focus()
  }
})

const togglePopover = () => {
  isOpen.value = !isOpen.value
}

const isVisible = computed(() => {
  const s = props.status?.toLowerCase()
  return s !== 'ready'
})

const isFailed = computed(() => {
  const s = props.status?.toLowerCase()
  return s === 'failed' || s === 'error'
})

const isRunning = computed(() => {
  const s = props.status?.toLowerCase()
  return s === 'processing' || s === 'running' || s === 'pending'
})

const formattedPhase = computed(() => {
  if (!props.currentPhase) return 'Initializing'
  return props.currentPhase
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
})

const phaseDescription = computed(() => {
  if (props.message) return props.message
  switch (props.currentPhase) {
    case 'preparing_input':
      return 'Validating candidate dossier and target role specifications'
    case 'analyzing_resume':
      return 'Extracting verifiable engineering capabilities and timeline from CV'
    case 'reading_job_description':
      return 'Parsing core requirements, stack profile, and level criteria'
    case 'researching_company':
      return 'Querying verified company intelligence and product landscape'
    case 'researching_engineering_environment':
      return 'Analyzing engineering blogs, tech stack choices, and architecture'
    case 'extracting_role_requirements':
      return 'Mapping structural requirements against domain knowledge'
    case 'mapping_resume_evidence':
      return 'Computing candidate strength matrix and gap analysis'
    case 'constructing_knowledge_graph':
      return 'Assembling multi-relational SurrealDB knowledge graph'
    case 'building_learning_plan':
      return 'Sequencing prerequisite-ordered technical modules'
    case 'generating_lessons':
      return 'Compiling deep technical lessons and topology diagrams'
    case 'building_interview_question_book':
      return 'Generating high-yield interview questions with answer outlines'
    case 'final_validation':
      return 'Executing semantic citation checks and score verification'
    default:
      return 'Asynchronous durable workflow processing'
  }
})
</script>

<template>
  <div v-if="isVisible" ref="popoverRef" class="relative inline-block text-left">

    <button
      ref="triggerRef"
      type="button"
      class="group inline-flex items-center gap-2 px-2.5 py-1 rounded-md text-xs font-medium transition-all border focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-1 dark:focus:ring-offset-slate-900"
      :class="[
        isFailed
          ? 'bg-rose-50 dark:bg-rose-950/40 border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 hover:bg-rose-100 dark:hover:bg-rose-900/50'
          : 'bg-indigo-50/80 dark:bg-indigo-950/50 border-indigo-200/80 dark:border-indigo-800/80 text-indigo-700 dark:text-indigo-300 hover:bg-indigo-100/80 dark:hover:bg-indigo-900/60',
        isOpen ? 'ring-2 ring-indigo-500/30' : '',
      ]"
      :aria-expanded="isOpen"
      aria-haspopup="dialog"
      :aria-label="`Pipeline status: ${formattedPhase}, ${progressPercent}% complete`"
      @click="togglePopover"
    >

      <span class="relative flex h-2 w-2 items-center justify-center">
        <span
          v-if="isRunning"
          class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
          :class="isFailed ? 'bg-rose-400' : 'bg-indigo-400'"
        />
        <span
          class="relative inline-flex rounded-full h-2 w-2"
          :class="isFailed ? 'bg-rose-500' : 'bg-indigo-600 dark:bg-indigo-400'"
        />
      </span>

      <span class="font-medium truncate max-w-[140px] sm:max-w-[180px]">
        {{ formattedPhase }}
      </span>

      <span
        v-if="!isFailed"
        class="font-mono tabular-nums text-[11px] px-1 py-0.2 rounded bg-indigo-200/50 dark:bg-indigo-900/70 text-indigo-800 dark:text-indigo-200"
      >
        {{ progressPercent }}%
      </span>

      <ChevronDown
        class="w-3.5 h-3.5 text-slate-400 transition-transform duration-150"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>


    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-80 sm:w-96 rounded-lg bg-white dark:bg-slate-900 shadow-xl border border-slate-200 dark:border-slate-800 p-4 z-50 text-left transition-all animate-in fade-in zoom-in-95 duration-150"
      role="dialog"
      aria-label="Durable Research Pipeline Status"
    >

      <div class="flex items-center justify-between pb-3 border-b border-slate-100 dark:border-slate-800">
        <div class="flex items-center gap-2">
          <Activity class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          <span class="text-xs font-semibold text-slate-900 dark:text-white uppercase tracking-wider font-mono">
            Durable Pipeline
          </span>
        </div>

        <div class="flex items-center gap-1.5 text-[11px] text-slate-500">
          <component
            :is="isConnected ? Wifi : WifiOff"
            class="w-3.5 h-3.5"
            :class="isConnected ? 'text-emerald-500' : 'text-slate-400'"
          />
          <span class="font-mono">{{ isConnected ? 'Live SSE' : 'Polling' }}</span>
        </div>
      </div>


      <div class="py-3 space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-slate-600 dark:text-slate-400 font-medium">
            Current Phase
          </span>
          <span class="font-semibold text-slate-900 dark:text-white">
            {{ formattedPhase }}
          </span>
        </div>


        <div class="w-full bg-slate-100 dark:bg-slate-800 rounded-full h-2 overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-500 ease-out"
            :class="isFailed ? 'bg-rose-500' : 'bg-indigo-600 dark:bg-indigo-500'"
            :style="{ width: `${Math.max(5, Math.min(100, progressPercent))}%` }"
          />
        </div>

        <div class="flex items-center justify-between text-[11px] font-mono text-slate-500">
          <span>Overall Progress</span>
          <span class="font-semibold text-slate-900 dark:text-slate-200 tabular-nums">
            {{ progressPercent }}%
          </span>
        </div>
      </div>


      <div class="p-2.5 rounded-md bg-slate-50 dark:bg-slate-800/60 border border-slate-100 dark:border-slate-800 text-xs text-slate-600 dark:text-slate-300 leading-relaxed mb-3">
        <p class="font-medium text-[11px] uppercase tracking-wider text-slate-400 dark:text-slate-500 font-mono mb-1">
          Activity Detail
        </p>
        <p>{{ phaseDescription }}</p>
      </div>


      <div class="pt-2 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs">
        <span class="text-[11px] text-slate-400">
          Autonomous Orchestration
        </span>
        <NuxtLink
          to="/workflows"
          class="inline-flex items-center gap-1 text-indigo-600 dark:text-indigo-400 hover:text-indigo-500 font-medium text-xs transition-colors"
          @click="isOpen = false"
        >
          <span>Execution Studio</span>
          <ExternalLink class="w-3 h-3" />
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
