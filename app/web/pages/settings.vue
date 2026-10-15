<script setup lang="ts">
import { useAuthService } from '~/features/auth/services/auth.service'
import { useApiClient } from '~/services/api.client'
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'default',
})

const { user, setup2fa, verify2fa, disable2fa } = useAuthService()
const { request } = useApiClient()

const activeTab = ref<'providers' | 'routing' | 'security'>('providers')

// Provider BYOK State
interface CombinedProvider {
  provider_key: string
  name: string
  base_url?: string
  active_key_source: 'user' | 'system' | 'none'
  is_user_override: boolean
  is_enabled: boolean
  allowed_models: string[]
}

const { data: providers, refresh: refreshProviders } = await useAsyncData('user-providers', async () => {
  return await request<CombinedProvider[]>('/api/v1/settings/llm/providers')
})

const selectedProvider = ref<string>('openai')
const providerApiKey = ref<string>('')
const providerBaseUrl = ref<string>('')
const isSavingProvider = ref<boolean>(false)
const providerFeedback = ref<string>('')

// Model Routing Matrix State
interface ModelSlot {
  provider_key: string
  model: string
  temperature?: number
}
interface SlotConfig {
  primary: ModelSlot
  fallback?: ModelSlot
}
interface RoutingResponse {
  slots: Record<string, SlotConfig>
}

const { data: routingData, refresh: refreshRouting } = await useAsyncData('user-routing', async () => {
  return await request<RoutingResponse>('/api/v1/settings/llm/routing')
})

const roles = ['router', 'planner', 'research', 'generation'] as const
const slotRoles = [
  { id: 'router', name: 'Workflow Router', desc: 'Fast intent routing & complexity classification' },
  { id: 'planner', name: 'Planner Agent', desc: 'DAG construction & research subtask scoping' },
  { id: 'research', name: 'Research Specialists', desc: 'Company, architecture & technical scrapers' },
  { id: 'generation', name: 'Synthesis & Generation', desc: 'Evidence evaluation, curriculum & question rubrics' },
]

const matrixState = ref<Record<string, SlotConfig>>({
  router: { primary: { provider_key: 'openai', model: 'gpt-4o-mini', temperature: 0.1 } },
  planner: { primary: { provider_key: 'openai', model: 'gpt-4o', temperature: 0.2 } },
  research: { primary: { provider_key: 'openai', model: 'gpt-4o-mini', temperature: 0.2 } },
  generation: { primary: { provider_key: 'openai', model: 'gpt-4o', temperature: 0.2 } },
})

watch(
  routingData,
  (val) => {
    if (val?.slots) {
      for (const r of roles) {
        if (val.slots[r]) {
          matrixState.value[r] = val.slots[r]
        }
      }
    }
  },
  { immediate: true }
)

const isSavingMatrix = ref<boolean>(false)
const matrixFeedback = ref<string>('')

const handleSaveProvider = async () => {
  isSavingProvider.value = true
  providerFeedback.value = ''
  try {
    await request('/api/v1/settings/llm/providers', {
      method: 'PUT',
      body: {
        provider_key: selectedProvider.value,
        api_key: providerApiKey.value.trim() || undefined,
        base_url: providerBaseUrl.value.trim() || undefined,
      },
    })
    providerFeedback.value = `Personal credentials for ${selectedProvider.value} saved.`
    providerApiKey.value = ''
    await refreshProviders()
  } catch (err: any) {
    providerFeedback.value = err?.data?.error?.message || err?.message || 'Failed to save provider key.'
  } finally {
    isSavingProvider.value = false
  }
}

const handleRevertProvider = async (providerKey: string) => {
  try {
    await request(`/api/v1/settings/llm/providers/${providerKey}`, {
      method: 'DELETE',
    })
    await refreshProviders()
  } catch (err: any) {
    providerFeedback.value = 'Failed to revert provider to platform default.'
  }
}

const handleSaveRoutingMatrix = async () => {
  isSavingMatrix.value = true
  matrixFeedback.value = ''
  try {
    await request('/api/v1/settings/llm/routing', {
      method: 'PUT',
      body: {
        slots: matrixState.value,
      },
    })
    matrixFeedback.value = 'Model routing matrix updated.'
    await refreshRouting()
  } catch (err: any) {
    matrixFeedback.value = err?.data?.error?.message || err?.message || 'Failed to update routing matrix.'
  } finally {
    isSavingMatrix.value = false
  }
}

// 2FA Security State
const totpSetupData = ref<{ secret: string; qr_code_svg: string; otpauth_url: string } | null>(null)
const totpCodeInput = ref<string>('')
const twoFaError = ref<string>('')

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
  try {
    await disable2fa()
  } catch {
    twoFaError.value = 'Failed to disable 2FA.'
  }
}
</script>

<template>
  <div class="max-w-5xl mx-auto py-8 px-4 sm:px-6">
    <div class="mb-8 border-b border-surface-border pb-5">
      <h1 class="text-2xl font-bold tracking-tight text-text-primary">Workspace & LLM Configuration</h1>
      <p class="text-xs text-text-muted mt-1">
        Manage BYOK platform overrides, model role slots, and account authentication security.
      </p>

      <div class="flex gap-2 mt-5">
        <button
          type="button"
          @click="activeTab = 'providers'"
          :class="[
            'px-4 py-2 text-xs font-semibold rounded-lg transition border cursor-pointer',
            activeTab === 'providers' ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-white'
          ]"
        >
          BYOK Model Credentials
        </button>
        <button
          type="button"
          @click="activeTab = 'routing'"
          :class="[
            'px-4 py-2 text-xs font-semibold rounded-lg transition border cursor-pointer',
            activeTab === 'routing' ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-white'
          ]"
        >
          Model Routing Matrix
        </button>
        <button
          type="button"
          @click="activeTab = 'security'"
          :class="[
            'px-4 py-2 text-xs font-semibold rounded-lg transition border cursor-pointer',
            activeTab === 'security' ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-white'
          ]"
        >
          Security & 2FA
        </button>
      </div>
    </div>

    <div v-if="activeTab === 'providers'" class="space-y-6">
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm">
        <h2 class="text-sm font-bold text-text-primary mb-1">Bring Your Own Key (BYOK) Precedence</h2>
        <p class="text-xs text-text-muted mb-4">
          When configured, personal keys take strict 100% precedence over shared platform admin keys.
        </p>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
          <div
            v-for="p in (providers || [])"
            :key="p.provider_key"
            @click="selectedProvider = p.provider_key"
            class="p-4 bg-surface-base border rounded-xl flex flex-col justify-between cursor-pointer transition shadow-xs"
            :class="selectedProvider === p.provider_key ? 'ring-2 ring-indigo-600 border-indigo-600 bg-indigo-50/20 dark:bg-indigo-950/20' : (p.is_user_override ? 'border-indigo-300 dark:border-indigo-800 bg-indigo-50/10' : 'border-surface-border hover:border-indigo-300 dark:hover:border-indigo-700')"
          >
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-text-primary tracking-wide uppercase">{{ p.name }}</span>
                <span
                  class="px-2 py-0.5 text-[10px] font-mono rounded font-semibold"
                  :class="p.is_user_override ? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/40 dark:text-indigo-300' : (p.active_key_source === 'system' ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-400' : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400')"
                >
                  {{ p.is_user_override ? 'Your Key Active' : (p.active_key_source === 'system' ? 'Platform Admin Key' : 'No Key Set') }}
                </span>
              </div>
              <p class="text-[11px] text-text-muted font-mono truncate">{{ p.base_url || 'Default Cloud Endpoint' }}</p>
            </div>

            <div class="mt-4 pt-2.5 border-t border-surface-border/60 flex items-center justify-between text-[11px]">
              <span class="font-medium" :class="selectedProvider === p.provider_key ? 'text-indigo-600 dark:text-indigo-400 font-semibold' : 'text-text-muted'">
                {{ selectedProvider === p.provider_key ? '● Currently Selected' : 'Click to Configure' }}
              </span>
              <button
                v-if="p.is_user_override"
                type="button"
                @click.stop="handleRevertProvider(p.provider_key)"
                class="text-danger hover:underline font-medium"
              >
                Remove Override
              </button>
            </div>
          </div>
        </div>

        <div class="p-5 bg-surface-base border border-surface-border rounded-xl space-y-4">
          <div class="flex items-center justify-between border-b border-surface-border/60 pb-3">
            <div>
              <h3 class="text-xs font-bold text-text-primary uppercase tracking-wider">
                Configure Credentials: <span class="text-indigo-600 dark:text-indigo-400">{{ (providers || []).find(p => p.provider_key === selectedProvider)?.name || selectedProvider }}</span>
              </h3>
              <p class="text-[11px] text-text-muted mt-0.5">Enter your personal API key or custom endpoint for this provider.</p>
            </div>
            <span class="px-2 py-0.5 text-[10px] font-mono bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded border border-slate-200 dark:border-slate-700">
              ID: {{ selectedProvider }}
            </span>
          </div>

          <form @submit.prevent="handleSaveProvider" class="space-y-4">
            <div v-if="providerFeedback" class="p-3 rounded-lg text-xs" :class="providerFeedback.includes('Failed') ? 'bg-danger/10 text-danger border border-danger/20' : 'bg-success/10 text-success border border-success/20'">
              {{ providerFeedback }}
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-text-muted mb-1">Personal API Key</label>
                <input
                  v-model="providerApiKey"
                  type="password"
                  placeholder="sk-... (Leave empty to keep current key)"
                  class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-xs font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-indigo-600"
                />
              </div>
              <div>
                <label class="block text-xs font-semibold text-text-muted mb-1">Custom Base URL (Optional)</label>
                <input
                  v-model="providerBaseUrl"
                  type="url"
                  placeholder="https://api.openai.com/v1"
                  class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-2 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-indigo-600"
                />
              </div>
            </div>

            <div class="flex items-center justify-between pt-2">
              <p class="text-[11px] text-text-muted">
                Saved keys are AES-256 encrypted at rest and never shared with other workspace users.
              </p>
              <button
                type="submit"
                :disabled="isSavingProvider"
                class="px-5 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg disabled:opacity-50 transition cursor-pointer shadow-sm"
              >
                {{ isSavingProvider ? 'Saving Key...' : 'Save Credentials' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'routing'" class="space-y-6">
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm">
        <div class="flex justify-between items-center mb-4">
          <div>
            <h2 class="text-sm font-bold text-text-primary">Functional Role Slots & Fallback Matrix</h2>
            <p class="text-xs text-text-muted">Configure dedicated primary and automatic fallback models per agent capability.</p>
          </div>
          <button
            type="button"
            @click="handleSaveRoutingMatrix"
            :disabled="isSavingMatrix"
            class="px-4 py-1.5 text-xs font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50 transition"
          >
            {{ isSavingMatrix ? 'Saving...' : 'Save Routing Matrix' }}
          </button>
        </div>

        <div v-if="matrixFeedback" class="mb-4 p-3 rounded-lg text-xs bg-success/10 text-success border border-success/20">
          {{ matrixFeedback }}
        </div>

        <div class="space-y-5">
          <div
            v-for="slot in slotRoles"
            :key="slot.id"
            class="p-4 bg-surface-base border border-surface-border rounded-lg space-y-3"
          >
            <div class="flex justify-between items-start">
              <div>
                <span class="text-xs font-mono font-bold text-primary uppercase">{{ slot.name }}</span>
                <p class="text-xs text-text-muted mt-0.5">{{ slot.desc }}</p>
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-surface-border/50">
              <div class="space-y-2">
                <span class="text-[11px] font-semibold text-text-primary">Primary Model</span>
                <div class="grid grid-cols-2 gap-2">
                  <select
                    v-model="matrixState[slot.id].primary.provider_key"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary cursor-pointer"
                  >
                    <option v-for="p in (providers || [])" :key="p.provider_key" :value="p.provider_key" class="bg-surface-base text-text-primary">
                      {{ p.name }}
                    </option>
                  </select>
                  <input
                    v-model="matrixState[slot.id].primary.model"
                    type="text"
                    placeholder="Model ID (e.g. gpt-4o)"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary"
                  />
                </div>
              </div>

              <div class="space-y-2">
                <span class="text-[11px] font-semibold text-text-muted">Fallback Model (Triggered on 5xx/429)</span>
                <div class="grid grid-cols-2 gap-2">
                  <select
                    v-if="matrixState[slot.id].fallback"
                    v-model="matrixState[slot.id].fallback!.provider_key"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary cursor-pointer"
                  >
                    <option v-for="p in (providers || [])" :key="p.provider_key" :value="p.provider_key" class="bg-surface-base text-text-primary">
                      {{ p.name }}
                    </option>
                  </select>
                  <input
                    v-if="matrixState[slot.id].fallback"
                    v-model="matrixState[slot.id].fallback!.model"
                    type="text"
                    placeholder="Fallback Model ID"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'security'" class="space-y-6">
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm space-y-4">
        <h2 class="text-sm font-bold text-text-primary">Two-Factor Authentication (TOTP)</h2>
        <p class="text-xs text-text-muted">
          Protect your account with RFC 6238 compliant hardware or software authenticators.
        </p>

        <div v-if="twoFaError" class="p-3 bg-danger/10 text-danger text-xs rounded-lg border border-danger/20">
          {{ twoFaError }}
        </div>

        <div v-if="user?.totp_enabled" class="flex items-center justify-between p-3 bg-success/10 border border-success/20 rounded-lg">
          <span class="text-xs font-medium text-success">2FA is currently active on your account.</span>
          <button
            type="button"
            @click="handleDisable2fa"
            class="px-3 py-1 bg-danger text-white rounded text-xs font-medium"
          >
            Disable 2FA
          </button>
        </div>

        <div v-else-if="totpSetupData" class="space-y-4 p-4 border border-surface-border rounded-lg">
          <div class="text-xs text-text-muted">Scan QR code with your authenticator app:</div>
          <div v-html="totpSetupData.qr_code_svg" class="w-48 h-48 bg-white p-2 rounded"></div>
          <div class="flex gap-2">
            <input
              v-model="totpCodeInput"
              type="text"
              placeholder="6-digit code"
              class="bg-surface-base border border-surface-border rounded-lg px-3 py-1.5 text-sm font-mono text-text-primary"
            />
            <button
              type="button"
              @click="handleVerify2fa"
              class="px-4 py-1.5 text-xs font-semibold bg-primary text-white rounded-lg"
            >
              Verify & Enable
            </button>
          </div>
        </div>

        <div v-else>
          <button
            type="button"
            @click="handleStart2faSetup"
            class="px-4 py-2 text-xs font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover transition"
          >
            Configure 2FA
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
