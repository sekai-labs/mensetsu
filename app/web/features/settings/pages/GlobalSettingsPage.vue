<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAuthService } from '~/features/auth/services/auth.service'
import type { LlmConfigItem, UpsertLlmConfigRequest } from '../models/settings.model'
import { useSettingsService } from '../services/settings.service'
import ProviderSidebar from '../components/ProviderSidebar.vue'
import LlmConfigForm from '../components/LlmConfigForm.vue'
import TotpSecuritySettings from '../components/TotpSecuritySettings.vue'
const { user } = useAuthService()
const settingsService = useSettingsService()

const {
  data: llmConfigs,
  refresh: refreshConfigs,
} = await useAsyncData<LlmConfigItem[]>(
  'llm-settings-list',
  async () => {
    try {
      return await settingsService.getLlmConfigs()
    } catch {
      return []
    }
  },
  { default: () => [] }
)

const activeProvider = ref<string>('openai')
const isSaving = ref<boolean>(false)
const saveSuccess = ref<boolean>(false)
const saveError = ref<string | null>(null)

const availableProviders = computed<LlmConfigItem[]>(() => {
  return Array.isArray(llmConfigs.value) ? llmConfigs.value : []
})

const currentConfig = computed<LlmConfigItem | undefined>(() => {
  return availableProviders.value.find(c => c.provider === activeProvider.value)
})

const isAdmin = computed<boolean>(() => {
  return user.value?.role === 'admin'
})

const handleSelectProvider = (provName: string): void => {
  activeProvider.value = provName
  saveSuccess.value = false
  saveError.value = null
}

const handleSaveConfig = async (payload: UpsertLlmConfigRequest): Promise<void> => {
  isSaving.value = true
  saveError.value = null
  saveSuccess.value = false

  try {
    await settingsService.saveLlmConfig(payload)
    saveSuccess.value = true
    await refreshConfigs()

    setTimeout(() => {
      saveSuccess.value = false
    }, 4000)
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'data' in err) {
      const dataObj = (err as { data?: { error?: { message?: string } } }).data
      saveError.value = dataObj?.error?.message || 'Failed to update LLM configuration'
    } else if (err instanceof Error) {
      saveError.value = err.message
    } else {
      saveError.value = 'Failed to update LLM configuration'
    }
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  if (availableProviders.value.length > 0) {
    const hasOpenAi = availableProviders.value.some(p => p.provider === 'openai')
    activeProvider.value = hasOpenAi ? 'openai' : availableProviders.value[0].provider
  }
})
</script>

<template>
  <div class="min-h-screen bg-slate-50 dark:bg-slate-950 font-sans">
    <header class="border-b border-slate-200 dark:border-slate-800 bg-white/90 dark:bg-slate-900/90 backdrop-blur sticky top-0 z-40">
      <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 h-14 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <NuxtLink to="/dashboard" class="flex items-center gap-2">
            <span class="w-6 h-6 rounded bg-indigo-600 flex items-center justify-center text-white text-xs font-bold">
              M
            </span>
            <span class="text-xs font-bold tracking-tight text-slate-900 dark:text-white">
              Mensetsu
            </span>
          </NuxtLink>
          <span class="text-slate-300 dark:text-slate-700">/</span>
          <span class="text-xs font-semibold text-slate-900 dark:text-white">Global Settings</span>
        </div>

        <NuxtLink
          to="/dashboard"
          class="px-3 py-1.5 text-xs font-medium rounded border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition"
        >
          ← Back to Dashboard
        </NuxtLink>
      </div>
    </header>

    <main class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
      <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2 mb-1">
            <span class="px-2 py-0.5 text-[10px] font-bold rounded bg-indigo-50 text-indigo-700 dark:bg-indigo-950 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800">
              AI INFRASTRUCTURE
            </span>
            <span v-if="isAdmin" class="px-2 py-0.5 text-[10px] font-bold rounded bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300 border border-amber-200 dark:border-amber-800">
              ADMIN MODE
            </span>
          </div>
          <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
            LLM Providers & Multi-Model Routing
          </h1>
          <p class="text-xs text-slate-500 mt-1 max-w-2xl">
            Configure private provider API credentials encrypted with AES-256-GCM at rest. Admin accounts can share models organization-wide to members.
          </p>
        </div>

        <div class="flex items-center gap-2 p-3 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300 text-xs font-mono">
          <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
          <span>AES-256-GCM Encrypted Storage</span>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-4 gap-6 items-start">
        <ProviderSidebar
          :providers="availableProviders"
          :active-provider="activeProvider"
          @select="handleSelectProvider"
        />

        <LlmConfigForm
          :active-provider="activeProvider"
          :current-config="currentConfig"
          :is-admin="isAdmin"
          :is-saving="isSaving"
          :save-success="saveSuccess"
          :save-error="saveError"
          @save="handleSaveConfig"
        />
      </div>

      <TotpSecuritySettings />
    </main>
  </div>
</template>
