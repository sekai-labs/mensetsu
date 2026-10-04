<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import type { LlmConfigItem, UpsertLlmConfigRequest } from '../models/settings.model'
import { resolveProviderMetadata, validateLlmConfigInput } from '../utils/settings.util'

const props = defineProps<{
  currentConfig?: LlmConfigItem
  activeProvider: string
  isAdmin?: boolean
  isSaving?: boolean
  saveSuccess?: boolean
  saveError?: string | null
}>()

const emit = defineEmits<{
  (e: 'save', payload: UpsertLlmConfigRequest): void
}>()

const editApiKey = ref<string>('')
const editBaseUrl = ref<string>('')
const editModelChat = ref<string>('')
const editModelSearch = ref<string>('')
const editModelEmbed = ref<string>('')
const editIsShared = ref<boolean>(false)
const showApiKey = ref<boolean>(false)
const localValidationErrors = ref<string[]>([])

const providerMeta = computed(() => {
  return resolveProviderMetadata(props.activeProvider)
})

const populateFields = () => {
  editApiKey.value = ''
  editBaseUrl.value = props.currentConfig?.base_url || ''
  editModelChat.value = props.currentConfig?.model_chat || ''
  editModelSearch.value = props.currentConfig?.model_search || ''
  editModelEmbed.value = props.currentConfig?.model_embed || ''
  editIsShared.value = props.currentConfig?.is_shared || false
  localValidationErrors.value = []
}

watch(
  () => props.activeProvider,
  () => {
    populateFields()
  },
  { immediate: true }
)

watch(
  () => props.currentConfig,
  () => {
    populateFields()
  }
)

const handleSubmit = (): void => {
  const payload: UpsertLlmConfigRequest = {
    provider: props.activeProvider,
    api_key: editApiKey.value.trim() ? editApiKey.value.trim() : undefined,
    base_url: editBaseUrl.value.trim() ? editBaseUrl.value.trim() : undefined,
    model_chat: editModelChat.value.trim() ? editModelChat.value.trim() : undefined,
    model_search: editModelSearch.value.trim() ? editModelSearch.value.trim() : undefined,
    model_embed: editModelEmbed.value.trim() ? editModelEmbed.value.trim() : undefined,
    is_shared: props.isAdmin ? editIsShared.value : false,
  }

  const validation = validateLlmConfigInput(payload)
  if (!validation.valid) {
    localValidationErrors.value = validation.errors.map(err => err.message)
    return
  }

  localValidationErrors.value = []
  emit('save', payload)
}
</script>

<template>
  <div class="md:col-span-3 bg-white dark:bg-slate-900 p-6 sm:p-8 rounded-lg border border-slate-200 dark:border-slate-800 shadow-sm space-y-6">
    <div class="flex items-center justify-between pb-4 border-b border-slate-200 dark:border-slate-800">
      <div>
        <h2 class="text-base font-bold text-slate-900 dark:text-white capitalize">
          {{ providerMeta.name }} Settings
        </h2>
        <p v-if="providerMeta.description" class="text-xs text-slate-500 mt-0.5">
          {{ providerMeta.description }}
        </p>
        <span v-if="props.currentConfig?.masked_key" class="text-xs font-mono text-slate-400 mt-1 block">
          Current Key: {{ props.currentConfig.masked_key }}
        </span>
      </div>

      <div
        v-if="props.currentConfig?.is_inherited"
        class="px-2.5 py-1 text-xs rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800/50 font-semibold"
      >
        Inherited from Admin Public Sharing
      </div>
    </div>

    <form class="space-y-4" @submit.prevent="handleSubmit">
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <label class="text-xs font-semibold text-slate-700 dark:text-slate-300">
            API Key
          </label>
          <button
            type="button"
            class="text-[11px] text-indigo-600 dark:text-indigo-400 font-semibold hover:underline"
            @click="showApiKey = !showApiKey"
          >
            {{ showApiKey ? 'Hide' : 'Show' }}
          </button>
        </div>
        <input
          v-model="editApiKey"
          :type="showApiKey ? 'text' : 'password'"
          :placeholder="props.currentConfig?.has_key ? 'Enter new key to replace existing...' : 'sk-...'"
          class="w-full px-3 py-2 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
        />
        <p class="text-[11px] text-slate-400">
          Key will be encrypted using 256-bit AES-GCM before saving to database.
        </p>
      </div>

      <div class="space-y-1.5">
        <label class="text-xs font-semibold text-slate-700 dark:text-slate-300">
          Base URL (Optional)
        </label>
        <input
          v-model="editBaseUrl"
          type="text"
          :placeholder="providerMeta.defaultBaseUrl || 'https://api.openai.com/v1 or http://localhost:11434/v1'"
          class="w-full px-3 py-2 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
        />
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-slate-700 dark:text-slate-300">
            Chat Model Override
          </label>
          <input
            v-model="editModelChat"
            type="text"
            :placeholder="providerMeta.defaultChatModel || 'e.g. gpt-4o, claude-3-7-sonnet'"
            class="w-full px-3 py-2 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-slate-700 dark:text-slate-300">
            Search / Synthesis Model
          </label>
          <input
            v-model="editModelSearch"
            type="text"
            :placeholder="providerMeta.defaultSearchModel || 'e.g. gpt-4o-mini, claude-3-5-haiku'"
            class="w-full px-3 py-2 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div v-if="props.isAdmin" class="p-4 rounded bg-amber-50/50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800/40 space-y-2">
        <div class="flex items-center gap-3">
          <input
            id="is-shared-toggle"
            v-model="editIsShared"
            type="checkbox"
            class="rounded text-indigo-600 focus:ring-indigo-500 h-4 w-4"
          />
          <label for="is-shared-toggle" class="text-xs font-bold text-slate-900 dark:text-white cursor-pointer">
            Share this provider configuration with all organization members
          </label>
        </div>
        <p class="text-[11px] text-slate-500 dark:text-slate-400">
          When enabled, organization members without personal keys will automatically route requests through this credential. Raw API keys are never exposed to members.
        </p>
      </div>

      <div v-if="localValidationErrors.length > 0" class="p-3 rounded bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-800 dark:text-rose-300 text-xs space-y-1">
        <div v-for="err in localValidationErrors" :key="err">
          {{ err }}
        </div>
      </div>

      <div v-if="props.saveSuccess" class="p-3 rounded bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300 text-xs font-semibold">
        Configuration successfully saved and encrypted.
      </div>

      <div v-if="props.saveError" class="p-3 rounded bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-800 dark:text-rose-300 text-xs font-semibold">
        {{ props.saveError }}
      </div>

      <div class="flex justify-end pt-2">
        <button
          type="submit"
          class="px-5 py-2 rounded bg-indigo-600 text-white text-xs font-semibold hover:bg-indigo-500 transition shadow-sm disabled:opacity-50"
          :disabled="props.isSaving"
        >
          {{ props.isSaving ? 'Saving...' : 'Save Provider Settings' }}
        </button>
      </div>
    </form>
  </div>
</template>
