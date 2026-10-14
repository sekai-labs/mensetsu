<script setup lang="ts">
import { useAuthService } from '~/features/auth/services/auth.service'
import { useSettingsService } from '~/features/settings/services/settings.service'
import { KNOWN_LLM_PROVIDERS } from '~/features/settings/utils/settings.util'
import type { LlmConfigItem, LlmProviderInfo } from '~/features/settings/models/settings.model'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const { user, setup2fa, verify2fa, disable2fa } = useAuthService()
const { listLlmConfigs, upsertLlmConfig } = useSettingsService()

const { data: llmConfigs, refresh: refreshLlm } = await useAsyncData('llm-settings', () => listLlmConfigs())

const activeTab = ref<'llm' | 'security'>('llm')
const selectedProviderId = ref<string>('openai')
const apiKeyInput = ref<string>('')
const baseUrlInput = ref<string>('')
const modelChatInput = ref<string>('')
const isSharedInput = ref<boolean>(false)
const isSaving = ref<boolean>(false)
const feedbackMessage = ref<string>('')


const totpSetupData = ref<{ secret: string; qr_code_svg: string; otpauth_url: string } | null>(null)
const totpCodeInput = ref<string>('')
const twoFaError = ref<string>('')

const currentProviderInfo = computed<LlmProviderInfo>(() => {
  return KNOWN_LLM_PROVIDERS.find((p) => p.id === selectedProviderId.value) || KNOWN_LLM_PROVIDERS[0]!
})

const currentConfigItem = computed<LlmConfigItem | undefined>(() => {
  return (llmConfigs.value || []).find((c) => c.provider === selectedProviderId.value)
})

const isAdmin = computed(() => user.value?.role === 'admin')

watch(
  selectedProviderId,
  () => {
    const existing = currentConfigItem.value
    apiKeyInput.value = ''
    baseUrlInput.value = existing?.base_url || currentProviderInfo.value.defaultBaseUrl || ''
    modelChatInput.value = existing?.model_chat || currentProviderInfo.value.defaultChatModel || ''

    isSharedInput.value = (isAdmin.value && !existing?.is_inherited) ? (existing?.is_shared || false) : false
  },
  { immediate: true }
)

const handleSaveLlm = async () => {
  isSaving.value = true
  feedbackMessage.value = ''
  try {
    await upsertLlmConfig({
      provider: selectedProviderId.value,
      api_key: apiKeyInput.value.trim() || undefined,
      base_url: baseUrlInput.value.trim() || undefined,
      model_chat: modelChatInput.value.trim() || undefined,
      is_shared: isAdmin.value ? isSharedInput.value : false,
    })
    feedbackMessage.value = `${currentProviderInfo.value.name} configuration saved successfully.`
    apiKeyInput.value = ''
    await refreshLlm()
  } catch (err: unknown) {
    feedbackMessage.value = 'Failed to save configuration.'
  } finally {
    isSaving.value = false
  }
}

const handleStart2faSetup = async () => {
  twoFaError.value = ''
  try {
    totpSetupData.value = await setup2fa()
  } catch {
    twoFaError.value = 'Failed to generate 2FA setup secret.'
  }
}

const handleVerify2fa = async () => {
  twoFaError.value = ''
  try {
    await verify2fa({ code: totpCodeInput.value.trim() })
    totpSetupData.value = null
    totpCodeInput.value = ''
  } catch {
    twoFaError.value = 'Invalid verification code.'
  }
}

const handleDisable2fa = async () => {
  twoFaError.value = ''
  const code = prompt('Enter 2FA code to confirm disabling:')
  if (!code) return
  try {
    await disable2fa({ code: code.trim() })
  } catch {
    twoFaError.value = 'Failed to disable 2FA.'
  }
}
</script>

<template>
  <div class="space-y-6 max-w-4xl">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        System & Account Settings
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
        Configure LLM inference providers, API keys, and account authentication security.
      </p>
    </div>


    <div class="border-b border-slate-200 dark:border-slate-800 flex items-center gap-2">
      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px"
        :class="activeTab === 'llm'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'llm'"
      >
        LLM Inference Providers
      </button>
      <button
        type="button"
        class="px-3 py-2 text-xs font-semibold border-b-2 transition-colors -mb-px"
        :class="activeTab === 'security'
          ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400'
          : 'border-transparent text-slate-500 hover:text-slate-900 dark:hover:text-white'"
        @click="activeTab = 'security'"
      >
        Security & 2FA
      </button>
    </div>


    <div v-if="activeTab === 'llm'" class="flex flex-col md:flex-row gap-6 items-start">

      <aside class="w-full md:w-56 shrink-0 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-2 shadow-sm space-y-1">
        <button
          v-for="prov in KNOWN_LLM_PROVIDERS"
          :key="prov.id"
          type="button"
          class="w-full text-left px-3 py-2 rounded text-xs font-medium transition-colors flex items-center justify-between"
          :class="selectedProviderId === prov.id
            ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-semibold'
            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-slate-900 dark:hover:text-white'"
          @click="selectedProviderId = prov.id"
        >
          <span>{{ prov.name }}</span>
          <span
            v-if="(llmConfigs || []).some((c) => c.provider === prov.id && c.has_key)"
            class="w-1.5 h-1.5 rounded-full bg-emerald-500"
          ></span>
        </button>
      </aside>


      <div class="flex-1 w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm space-y-6">
        <div>
          <h2 class="text-base font-semibold text-slate-900 dark:text-white">
            {{ currentProviderInfo.name }} Configuration
          </h2>
          <p class="text-xs text-slate-500 mt-0.5">
            {{ currentProviderInfo.description }}
          </p>
        </div>

        <div
          v-if="feedbackMessage"
          class="p-3 text-xs rounded-md bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300"
        >
          {{ feedbackMessage }}
        </div>

        <form class="space-y-4" @submit.prevent="handleSaveLlm">
          <div class="space-y-1.5">
            <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
              API Secret Key
            </label>
            <input
              v-model="apiKeyInput"
              type="password"
              :placeholder="currentConfigItem?.has_key ? (currentConfigItem.masked_key || '••••••••') : 'Enter API Key'"
              class="block w-full rounded-md border text-xs px-3 py-2 font-mono bg-white dark:bg-slate-900 border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
            />
            <p v-if="currentConfigItem?.has_key" class="text-[11px] text-emerald-600 dark:text-emerald-400 font-mono">
              ✓ Active key configured ({{ currentConfigItem.masked_key || 'Masked' }})
            </p>
          </div>

          <AppInput
            id="base-url"
            v-model="baseUrlInput"
            label="Base URL / API Endpoint"
            placeholder="https://api.openai.com/v1"
          />

          <AppInput
            id="chat-model"
            v-model="modelChatInput"
            label="Default Chat / Reasoning Model"
            placeholder="e.g. gpt-4o, claude-3-5-sonnet-latest"
          />



          <div v-if="isAdmin" class="flex items-start gap-3 p-3 bg-slate-50 dark:bg-slate-800/50 rounded-md border border-slate-200 dark:border-slate-800">
            <input
              id="is-shared-toggle"
              v-model="isSharedInput"
              type="checkbox"
              class="mt-0.5 h-4 w-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-500 dark:border-slate-700 dark:bg-slate-900"
            />
            <label for="is-shared-toggle" class="text-xs select-none cursor-pointer">
              <span class="font-medium text-slate-800 dark:text-slate-200 block">Share organization-wide</span>
              <span class="text-slate-500 dark:text-slate-400 block mt-0.5">
                Allow team members who have not configured their own keys to inherit this configuration.
              </span>
            </label>
          </div>
          <div v-else class="text-xs text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-slate-800/30 p-2.5 rounded-md border border-slate-200 dark:border-slate-800 flex items-center gap-2">
            <span class="inline-block w-2 h-2 rounded-full bg-slate-400"></span>
            <span>Personal key (saved to your private profile).</span>
          </div>
          <div class="pt-2 flex items-center justify-between">
            <AppButton type="submit" size="sm" :loading="isSaving">
              Save {{ currentProviderInfo.name }} Settings
            </AppButton>
          </div>
        </form>
      </div>
    </div>


    <div v-else class="max-w-xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm space-y-6">
      <div>
        <h2 class="text-base font-semibold text-slate-900 dark:text-white">
          Two-Factor Authentication (TOTP)
        </h2>
        <p class="text-xs text-slate-500 mt-0.5">
          Protect your account with RFC 6238 time-based one-time password security.
        </p>
      </div>

      <div
        v-if="twoFaError"
        class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300"
      >
        {{ twoFaError }}
      </div>

      <div v-if="user?.totp_enabled" class="space-y-4">
        <div class="p-3 bg-emerald-50 dark:bg-emerald-950/50 rounded border border-emerald-200 dark:border-emerald-800 text-xs text-emerald-800 dark:text-emerald-200 flex items-center justify-between">
          <span>✓ Two-Factor Authentication is currently ENABLED.</span>
          <button
            type="button"
            class="text-xs text-rose-600 dark:text-rose-400 hover:underline font-semibold"
            @click="handleDisable2fa"
          >
            Disable 2FA
          </button>
        </div>
      </div>

      <div v-else class="space-y-4">
        <div v-if="!totpSetupData">
          <p class="text-xs text-slate-600 dark:text-slate-400 mb-3">
            Two-factor authentication adds an extra layer of security to your account during login.
          </p>
          <AppButton size="sm" @click="handleStart2faSetup">
            Enable 2FA Authentication
          </AppButton>
        </div>

        <div v-else class="space-y-4 pt-2">
          <div class="text-xs font-mono bg-slate-50 dark:bg-slate-850 p-3 rounded border border-slate-200 dark:border-slate-700">
            <span class="block text-slate-400 mb-1">Manual Setup Secret:</span>
            <span class="font-bold select-all text-indigo-600 dark:text-indigo-400">{{ totpSetupData.secret }}</span>
          </div>

          <form class="space-y-3" @submit.prevent="handleVerify2fa">
            <AppInput
              id="totp-code"
              v-model="totpCodeInput"
              label="Enter 6-Digit Authenticator Code"
              placeholder="123456"
              required
            />
            <AppButton type="submit" size="sm">
              Verify & Activate 2FA
            </AppButton>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>
