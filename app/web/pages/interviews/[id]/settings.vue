<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace } = useInterviewService()

const { data: ws } = await useAsyncData(
  `ws-settings-${workspaceId.value}`,
  () => getWorkspace(workspaceId.value)
)

const companyName = ref(ws.value?.company_name || '')
const roleTitle = ref(ws.value?.role_title || '')
const isSaved = ref(false)

const handleSave = () => {
  isSaved.value = true
  setTimeout(() => {
    isSaved.value = false
  }, 3000)
}
</script>

<template>
  <div class="space-y-6 max-w-xl">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
        <span>WORKSPACE CONFIGURATION</span>
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        Workspace Settings
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
        Manage metadata, target timeline, and preparation focus areas.
      </p>
    </div>

    <div
      v-if="isSaved"
      class="p-3 text-xs rounded-md bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300"
    >
      Workspace parameters updated successfully.
    </div>

    <form class="space-y-4" @submit.prevent="handleSave">
      <AppInput
        id="ws-company"
        v-model="companyName"
        label="Target Company Name"
        disabled
      />

      <AppInput
        id="ws-role"
        v-model="roleTitle"
        label="Target Role Title"
        disabled
      />

      <div class="space-y-1.5">
        <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
          Seniority Level
        </label>
        <input
          :value="ws?.seniority || 'Staff'"
          disabled
          class="block w-full rounded-md border text-xs px-3 py-2 bg-slate-50 dark:bg-slate-850 text-slate-500 border-slate-200 dark:border-slate-700"
        />
      </div>

      <div class="pt-2">
        <AppButton type="submit" size="sm">
          Save Changes
        </AppButton>
      </div>
    </form>
  </div>
</template>
