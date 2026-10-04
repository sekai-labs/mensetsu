<script setup lang="ts">
import { computed } from 'vue'
import type { AdminPipelinePhaseStep, StepState } from '../models/admin.model'
import {
  DEFAULT_PIPELINE_STEPS,
  calculateStepState,
  getStepCardStyle,
} from '../utils/admin.util'

const props = withDefaults(
  defineProps<{
    currentPhase: string
    status: string
    progressPercent?: number
    steps?: readonly AdminPipelinePhaseStep[]
  }>(),
  {
    progressPercent: 0,
    steps: () => DEFAULT_PIPELINE_STEPS,
  }
)

const stepList = computed(() => props.steps || DEFAULT_PIPELINE_STEPS)

const getStepStatus = (stepId: string): StepState => {
  return calculateStepState(stepId, props.currentPhase, props.status, stepList.value)
}

const getCardStyle = (stepId: string): string => {
  const state = getStepStatus(stepId)
  return getStepCardStyle(state)
}
</script>

<template>
  <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
          Pipeline Execution Stepper
        </h2>
        <span class="text-[11px] font-mono text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/60 px-2 py-0.5 rounded border border-indigo-200 dark:border-indigo-800 font-semibold">
          Phase: {{ currentPhase.replace(/_/g, ' ') }}
        </span>
      </div>
      <span class="text-xs font-mono font-semibold text-slate-600 dark:text-slate-300">
        {{ progressPercent }}% completed
      </span>
    </div>

    <!-- Stepper Grid -->
    <div class="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-2">
      <div
        v-for="(step, idx) in stepList"
        :key="step.id"
        class="p-3 rounded border text-center transition flex flex-col justify-between"
        :class="getCardStyle(step.id)"
      >
        <div class="text-[10px] font-mono mb-1 text-slate-400 dark:text-slate-500">
          Step {{ idx + 1 }}
        </div>
        <div class="text-[11px] leading-tight font-medium my-auto">
          {{ step.name }}
        </div>
        <div class="mt-2 text-[9px] font-mono uppercase tracking-wider font-semibold">
          {{ getStepStatus(step.id) }}
        </div>
      </div>
    </div>
  </div>
</template>
