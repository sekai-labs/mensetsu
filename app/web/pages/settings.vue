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
  <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:py-9">
    <header class="mb-7 border-b border-surface-border pb-5">
      <h1 class="text-2xl font-semibold tracking-tight text-text-primary">Settings</h1>
      <p class="mt-1 max-w-2xl text-sm text-text-muted">
        Configure model providers, workflow routing, and account security.
      </p>
    </header>

    <div class="grid gap-8 lg:grid-cols-[220px_minmax(0,1fr)] lg:gap-10">
      <nav aria-label="Settings sections" class="self-start lg:sticky lg:top-6">
        <p class="mb-2 px-3 text-[11px] font-semibold uppercase tracking-wide text-text-muted">Workspace</p>
        <div class="flex gap-1 overflow-x-auto pb-2 lg:flex-col lg:overflow-visible">
          <button type="button" @click="activeTab = 'providers'" :aria-current="activeTab === 'providers' ? 'page' : undefined" :class="['min-h-10 shrink-0 rounded-md px-3 text-left text-sm transition', activeTab === 'providers' ? 'bg-surface-elevated font-medium text-text-primary' : 'text-text-muted hover:bg-surface-elevated hover:text-text-primary']">Provider credentials</button>
          <button type="button" @click="activeTab = 'routing'" :aria-current="activeTab === 'routing' ? 'page' : undefined" :class="['min-h-10 shrink-0 rounded-md px-3 text-left text-sm transition', activeTab === 'routing' ? 'bg-surface-elevated font-medium text-text-primary' : 'text-text-muted hover:bg-surface-elevated hover:text-text-primary']">Model routing</button>
          <button type="button" @click="activeTab = 'security'" :aria-current="activeTab === 'security' ? 'page' : undefined" :class="['min-h-10 shrink-0 rounded-md px-3 text-left text-sm transition', activeTab === 'security' ? 'bg-surface-elevated font-medium text-text-primary' : 'text-text-muted hover:bg-surface-elevated hover:text-text-primary']">Security &amp; 2FA</button>
        </div>
      </nav>
      <main class="min-w-0">
    <div v-if="activeTab === 'providers'" class="space-y-6">
      <div class="space-y-5">
        <div class="border-b border-surface-border pb-4">
          <h2 class="text-base font-semibold text-text-primary">Provider credentials</h2>
          <p class="mt-1 text-sm text-text-muted">Personal keys take precedence over shared platform keys when configured.</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
          <div v-for="p in (providers || [])" :key="p.provider_key"
            class="rounded-md border border-surface-border bg-surface-base p-3 transition hover:border-slate-400 dark:hover:border-slate-600"
            :class="selectedProvider === p.provider_key ? 'border-slate-500 bg-surface-elevated ring-1 ring-slate-400/50 dark:ring-slate-600/50' : ''"
          >
            <button
              type="button"
              @click="selectedProvider = p.provider_key"
              :aria-pressed="selectedProvider === p.provider_key"
              class="block min-h-14 w-full text-left"
            >
              <span class="flex items-center justify-between gap-2">
                <span class="text-xs font-semibold text-text-primary">{{ p.name }}</span>
                <span
                  class="rounded px-2 py-0.5 text-[10px] font-medium"
                  :class="p.is_user_override ? 'bg-emerald-900/20 text-emerald-700 dark:text-emerald-300' : (p.active_key_source === 'system' ? 'bg-slate-200 text-slate-700 dark:bg-slate-800 dark:text-slate-300' : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400')"
                >
                  {{ p.is_user_override ? 'Personal key' : (p.active_key_source === 'system' ? 'Platform key' : 'No key') }}
                </span>
              </span>
              <span class="mt-2 block truncate font-mono text-[11px] text-text-muted">{{ p.base_url || 'Default cloud endpoint' }}</span>
              <span class="mt-2 block text-[11px] text-text-muted">{{ selectedProvider === p.provider_key ? 'Selected' : 'Configure' }}</span>
            </button>
            <div v-if="p.is_user_override" class="mt-2 border-t border-surface-border/60 pt-2 text-right">
              <button type="button" @click="handleRevertProvider(p.provider_key)" class="min-h-8 text-[11px] font-medium text-danger hover:underline">
                Remove override
              </button>
            </div>
          </div>
        </div>

        <div class="border-y border-surface-border py-4">
          <div class="flex flex-wrap items-center justify-between gap-3 border-b border-surface-border/60 pb-3">
            <div>
              <h3 class="text-sm font-semibold text-text-primary">
                Configure credentials: {{ (providers || []).find(p => p.provider_key === selectedProvider)?.name || selectedProvider }}
              </h3>
              <p class="mt-1 text-xs text-text-muted">Enter a personal API key or custom endpoint for this provider.</p>
            </div>
            <span class="rounded bg-surface-elevated px-2 py-1 font-mono text-[11px] text-text-muted">
              {{ selectedProvider }}
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
                  class="w-full rounded-md border border-surface-border bg-surface-base px-3 py-2 text-sm font-mono text-text-primary focus-visible:ring-2 focus-visible:ring-slate-500"
                />
              </div>
              <div>
                <label class="block text-xs font-semibold text-text-muted mb-1">Custom Base URL (Optional)</label>
                <input
                  v-model="providerBaseUrl"
                  type="url"
                  placeholder="https://api.openai.com/v1"
                  class="w-full rounded-md border border-surface-border bg-surface-base px-3 py-2 text-sm text-text-primary focus-visible:ring-2 focus-visible:ring-slate-500"
                />
              </div>
            </div>

              <p class="text-[11px] text-text-muted">
                Saved keys are AES-256 encrypted at rest and never shared with other workspace users.
              </p>
              <button
                type="submit"
                :disabled="isSavingProvider"
                class="min-h-10 rounded-md bg-slate-800 px-4 text-sm font-medium text-white transition hover:bg-slate-700 disabled:opacity-50 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-white"
              >
                {{ isSavingProvider ? 'Saving Key...' : 'Save Credentials' }}
              </button>
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
</main>
</div>
</div>
</template>
