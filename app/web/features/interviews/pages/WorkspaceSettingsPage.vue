<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace } = useInterviewService()
const { data: workspace } = await useAsyncData(
  `settings-ws-${workspaceId.value}`,
  () => getWorkspace(workspaceId.value)
)

const prepTimeWeeks = ref(workspace.value?.prep_time_weeks ?? 3)
const targetDate = ref(workspace.value?.target_date ?? '')
const isSaving = ref(false)
const savedSuccess = ref(false)

const handleSaveSettings = () => {
  isSaving.value = true
  setTimeout(() => {
    isSaving.value = false
    savedSuccess.value = true
    setTimeout(() => {
      savedSuccess.value = false
    }, 3000)
  }, 400)
}
</script>

<template>
  <div class="space-y-6 max-w-4xl">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
          Workspace Parameters & Model Allocation
        </h1>
        <p class="text-xs text-slate-500 mt-1">
          Adjust schedule constraints, target interview milestones, or route through custom LLM keys.
        </p>
      </div>

      <NuxtLink
        to="/settings"
        class="px-3 py-1.5 rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 text-xs font-semibold hover:bg-indigo-100 transition flex items-center gap-1.5"
      >
        <span>⚙ Configure Provider Keys</span>
        <span>→</span>
      </NuxtLink>
    </div>

    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-6">
      <h2 class="text-sm font-bold text-slate-900 dark:text-white">Preparation Constraints</h2>

      <div class="space-y-4">
        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1">
            Preparation Timeline (Weeks)
          </label>
          <input
            v-model="prepTimeWeeks"
            type="number"
            min="1"
            max="12"
            class="w-full sm:w-48 px-3 py-2 rounded-md border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
          <p class="text-[11px] text-slate-400 mt-1">Order curriculum units into this study schedule.</p>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1">
            Target Interview Date
          </label>
          <input
            v-model="targetDate"
            type="date"
            class="w-full sm:w-64 px-3 py-2 rounded-md border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div v-if="savedSuccess" class="p-3 rounded bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300 text-xs font-semibold">
          Workspace parameters successfully updated.
        </div>

        <div class="pt-2">
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
            :disabled="isSaving"
            @click="handleSaveSettings"
          >
            {{ isSaving ? 'Saving...' : 'Save Parameters' }}
          </button>
        </div>
      </div>
    </div>

    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-3">
      <h2 class="text-sm font-bold text-slate-900 dark:text-white">Workspace Information</h2>
      <div class="grid grid-cols-2 gap-4 text-xs font-mono text-slate-600 dark:text-slate-400">
        <div><strong>Workspace ID:</strong> {{ workspace?.id }}</div>
        <div><strong>Status:</strong> {{ workspace?.status }}</div>
        <div><strong>Target Company:</strong> {{ workspace?.company_name }}</div>
        <div><strong>Target Role:</strong> {{ workspace?.role_title }}</div>
      </div>
    </div>
  </div>
</template>
