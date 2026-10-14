<script setup lang="ts">
import { ref } from 'vue'
import { X, Play, Loader2, Sparkles, Workflow } from 'lucide-vue-next'
import type { CreateWorkflowRunPayload } from '~/services/workflow.service'

interface WorkspaceOption {
  id: string
  company_name: string
  role_title: string
  created_at?: string
}

const props = defineProps<{
  isOpen: boolean
  workspaces: WorkspaceOption[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'create', payload: CreateWorkflowRunPayload): void
}>()

const selectedType = ref<'static' | 'dynamic'>('static')
const workflowId = ref('interview-preparation-static')
const selectedWorkspaceId = ref<string>('')
const isSubmitting = ref(false)
const validationError = ref<string | null>(null)

const handleTypeChange = (type: 'static' | 'dynamic') => {
  selectedType.value = type
  if (type === 'static') {
    workflowId.value = 'interview-preparation-static'
  } else {
    workflowId.value = 'interview-preparation-dynamic'
  }
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleDateString()
}

const handleSubmit = async () => {
  validationError.value = null
  if (!selectedWorkspaceId.value) {
    validationError.value = 'Please select a target workspace to initialize the workflow run.'
    return
  }

  if (isSubmitting.value) return
  isSubmitting.value = true

  const payload: CreateWorkflowRunPayload = {
    workflow_id: workflowId.value.trim(),
    workspace_id: selectedWorkspaceId.value,
  }

  emit('create', payload)
  isSubmitting.value = false
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4"
    role="dialog"
    aria-modal="true"
    @click.self="emit('close')"
  >
    <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg shadow-xl max-w-lg w-full overflow-hidden">
      <div class="p-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-slate-50/50 dark:bg-slate-900/50">
        <div class="flex items-center gap-2">
          <Workflow class="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
          <h2 class="text-sm font-semibold text-slate-900 dark:text-white">
            Initialize Workflow Run
          </h2>
        </div>
        <button
          class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded"
          @click="emit('close')"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <div v-if="workspaces.length === 0" class="p-6 text-center space-y-4">
        <div class="w-12 h-12 rounded-full bg-indigo-50 dark:bg-indigo-950/50 flex items-center justify-center mx-auto text-indigo-600 dark:text-indigo-400">
          <Workflow class="w-6 h-6" />
        </div>
        <div class="space-y-1">
          <h3 class="text-sm font-semibold text-slate-900 dark:text-white">
            No Workspace Available
          </h3>
          <p class="text-xs text-slate-500 max-w-sm mx-auto">
            Workflows require an existing workspace containing your target company, role, CV, and JD.
          </p>
        </div>
        <NuxtLink
          to="/interviews/new"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded text-xs font-semibold shadow-xs transition-colors"
          @click="emit('close')"
        >
          Create New Workspace →
        </NuxtLink>
      </div>

      <form v-else class="p-5 space-y-4" @submit.prevent="handleSubmit">
        <div v-if="validationError" class="p-2.5 rounded bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-xs text-rose-700 dark:text-rose-400 font-medium">
          {{ validationError }}
        </div>

        <div class="space-y-1.5">
          <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
            Target Interview Workspace
          </label>
          <select
            v-model="selectedWorkspaceId"
            required
            class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded px-3 py-2 text-xs text-slate-900 dark:text-white font-mono"
          >
            <option value="" disabled>Select target workspace...</option>
            <option v-for="ws in workspaces" :key="ws.id" :value="ws.id">
              {{ ws.company_name }} · {{ ws.role_title }}{{ ws.created_at ? ` · ${formatDate(ws.created_at)}` : '' }}
            </option>
          </select>
        </div>

        <div class="space-y-1.5">
          <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
            Workflow Architecture
          </label>
          <div class="grid grid-cols-2 gap-3">
            <button
              type="button"
              class="p-3 border rounded-lg text-left transition-all flex flex-col gap-1"
              :class="selectedType === 'static' ? 'border-indigo-600 dark:border-indigo-500 bg-indigo-50/20 dark:bg-indigo-950/20 ring-1 ring-indigo-500' : 'border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700'"
              @click="handleTypeChange('static')"
            >
              <span class="text-xs font-semibold text-slate-900 dark:text-white">Static Pipeline</span>
              <span class="text-[11px] text-slate-500 leading-tight">Deterministic DAG execution with sequential and parallel phase transitions.</span>
            </button>

            <button
              type="button"
              class="p-3 border rounded-lg text-left transition-all flex flex-col gap-1"
              :class="selectedType === 'dynamic' ? 'border-indigo-600 dark:border-indigo-500 bg-indigo-50/20 dark:bg-indigo-950/20 ring-1 ring-indigo-500' : 'border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700'"
              @click="handleTypeChange('dynamic')"
            >
              <div class="flex items-center gap-1">
                <Sparkles class="w-3.5 h-3.5 text-amber-500" />
                <span class="text-xs font-semibold text-slate-900 dark:text-white">Dynamic Autonomous</span>
              </div>
              <span class="text-[11px] text-slate-500 leading-tight">Adaptive multi-agent execution with plan expansion and runtime delegation.</span>
            </button>
          </div>
        </div>

        <div class="pt-3 border-t border-slate-200 dark:border-slate-800 flex items-center justify-end gap-2">
          <button
            type="button"
            class="px-3 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded font-medium"
            @click="emit('close')"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="isSubmitting"
            class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-700 text-white rounded font-medium shadow-xs flex items-center gap-1.5 disabled:opacity-50"
          >
            <Loader2 v-if="isSubmitting" class="w-3.5 h-3.5 animate-spin" />
            <Play v-else class="w-3.5 h-3.5" />
            <span>Launch Run</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
