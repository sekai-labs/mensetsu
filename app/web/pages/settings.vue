<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthService } from '~/features/auth/services/auth.service'
import { useApiClient } from '~/services/api.client'
import type { OAuthIdentityDto, OAuthProviderSummaryDto, TotpSetupResponse } from '~/features/auth/models/auth.model'
import AppButton from '~/components/ui/AppButton.vue'
import { safeExternalHttpUrl } from '~/utils/safe-external-url'
import TokenUsageView from '~/components/usage/TokenUsageView.vue'

definePageMeta({
  layout: 'default',
})

const { t, te } = useI18n()
const route = useRoute()
const {
  user,
  setup2fa,
  verify2fa,
  disable2fa,
  fetchPublicOAuthProviders,
  fetchLinkedOAuthAccounts,
  initiateOAuthLink,
  unlinkOAuthAccount,
} = useAuthService()
const { request } = useApiClient()

type SettingsTab = 'providers' | 'routing' | 'security' | 'extension' | 'usage'
const activeTab = ref<SettingsTab>('providers')

const validTabs: SettingsTab[] = ['providers', 'routing', 'security', 'extension', 'usage']
if (route.query.tab && validTabs.includes(route.query.tab as SettingsTab)) {
  activeTab.value = route.query.tab as SettingsTab
}

watch(
  () => route.query.tab,
  (tab) => {
    if (tab && validTabs.includes(tab as SettingsTab)) {
      activeTab.value = tab as SettingsTab
    }
  }
)

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

const { data: routingMatrix, refresh: refreshRouting } = await useAsyncData('user-routing', async () => {
  return await request<Record<string, SlotConfig>>('/api/v1/settings/llm/routing')
})

const matrixState = ref<Record<string, SlotConfig>>({})
const isSavingMatrix = ref(false)
const matrixFeedback = ref('')

watch(
  routingMatrix,
  (val) => {
    if (val) {
      matrixState.value = JSON.parse(JSON.stringify(val))
    }
  },
  { immediate: true }
)

const handleSaveProviderKey = async () => {
  if (!providerApiKey.value.trim() && !providerBaseUrl.value.trim()) return
  isSavingProvider.value = true
  providerFeedback.value = ''
  try {
    await request(`/api/v1/settings/llm/providers/${encodeURIComponent(selectedProvider.value)}`, {
      method: 'PUT',
      body: {
        provider_key: selectedProvider.value,
        api_key: providerApiKey.value.trim() ? providerApiKey.value : undefined,
        base_url: providerBaseUrl.value.trim() || undefined,
      },
    })
    providerApiKey.value = ''
    providerFeedback.value = 'Provider configuration saved successfully.'
    await refreshProviders()
  } catch (err: any) {
    providerFeedback.value = err?.data?.error?.message || err?.message || 'Failed to save configuration.'
  } finally {
    isSavingProvider.value = false
  }
}

const handleClearProviderKey = async (providerKey: string) => {
  try {
    await request(`/api/v1/settings/llm/providers/${encodeURIComponent(providerKey)}`, {
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

// 2FA state & handling
const totpSetupData = ref<TotpSetupResponse | null>(null)
const totpCodeInput = ref<string>('')
const twoFaError = ref<string>('')
const isDisabling2fa = ref(false)
const totpDisableCode = ref('')
const isSubmittingDisable = ref(false)

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
  if (!totpDisableCode.value.trim()) {
    twoFaError.value = 'Please enter your 6-digit verification code.'
    return
  }
  isSubmittingDisable.value = true
  try {
    await disable2fa({ code: totpDisableCode.value.trim() })
    totpDisableCode.value = ''
    isDisabling2fa.value = false
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      twoFaError.value = err.message
    } else {
      twoFaError.value = 'Failed to disable 2FA. Verify your 6-digit code is correct.'
    }
  } finally {
    isSubmittingDisable.value = false
  }
}

// OAuth linked identities state & handling
const { data: publicOAuthProviders } = await useAsyncData('user-public-oauth', async () => {
  return await fetchPublicOAuthProviders()
})

const { data: linkedOAuthAccounts, refresh: refreshLinkedAccounts } = await useAsyncData(
  'user-linked-oauth',
  async () => {
    return await fetchLinkedOAuthAccounts()
  }
)

const oauthFeedback = ref('')
const oauthError = ref('')
const isLinkingProvider = ref<string | null>(null)
const isUnlinkingProvider = ref<string | null>(null)

onMounted(() => {
  if (route.query.oauth_linked === 'true') {
    oauthFeedback.value = t('oauth.linkedSuccess')
  }
  if (route.query.oauth_error) {
    const code = String(route.query.oauth_error)
    if (te(`oauth.errors.${code}`)) {
      oauthError.value = t(`oauth.errors.${code}`)
    } else {
      oauthError.value = t('oauth.errors.oauth_failed')
    }
  }
})

const getLinkedIdentity = (providerKey: string): OAuthIdentityDto | undefined => {
  return (linkedOAuthAccounts.value || []).find(
    (a) => a.provider_key.toLowerCase() === providerKey.toLowerCase()
  )
}

const handleLinkOAuth = async (providerKey: string) => {
  oauthFeedback.value = ''
  oauthError.value = ''
  isLinkingProvider.value = providerKey
  try {
    const res = await initiateOAuthLink(providerKey)
    const authorizationUrl = safeExternalHttpUrl(res.authorization_url)
    if (!authorizationUrl) throw new Error('OAuth provider returned an invalid authorization URL.')
    window.location.href = authorizationUrl
  } catch (err: any) {
    oauthError.value = err?.data?.error?.message || err?.message || 'Failed to initiate linking'
    isLinkingProvider.value = null
  }
}

const handleUnlinkOAuth = async (account: OAuthIdentityDto) => {
  oauthFeedback.value = ''
  oauthError.value = ''
  const confirmMsg = t('oauth.confirmDisconnect')
  if (!confirm(confirmMsg)) return

  isUnlinkingProvider.value = account.provider_key
  try {
    await unlinkOAuthAccount(account.id)
    oauthFeedback.value = t('oauth.unlinkSuccess')
    await refreshLinkedAccounts()
  } catch (err: any) {
    const msg = err?.data?.error?.message || err?.message || ''
    if (
      err?.statusCode === 400 ||
      err?.statusCode === 409 ||
      msg.includes('only') ||
      msg.includes('sole') ||
      msg.includes('last')
    ) {
      oauthError.value = t('oauth.errors.last_sign_in_method')
    } else {
      oauthError.value = msg || 'Failed to unlink account'
    }
  } finally {
    isUnlinkingProvider.value = null
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
          <button type="button" @click="activeTab = 'usage'" :aria-current="activeTab === 'usage' ? 'page' : undefined" :class="['min-h-10 shrink-0 rounded-md px-3 text-left text-sm transition', activeTab === 'usage' ? 'bg-surface-elevated font-medium text-text-primary' : 'text-text-muted hover:bg-surface-elevated hover:text-text-primary']">Token usage</button>
          <button type="button" @click="activeTab = 'extension'" :aria-current="activeTab === 'extension' ? 'page' : undefined" :class="['min-h-10 shrink-0 rounded-md px-3 text-left text-sm transition', activeTab === 'extension' ? 'bg-surface-elevated font-medium text-text-primary' : 'text-text-muted hover:bg-surface-elevated hover:text-text-primary']">Browser extension</button>
        </div>
      </nav>
      <main class="min-w-0">
    <div v-if="activeTab === 'extension'" class="space-y-6">
      <section class="rounded-xl border border-surface-border bg-surface-elevated/70 p-5 shadow-sm">
        <h2 class="text-sm font-bold text-text-primary">Install the browser extension</h2>
        <p class="mt-2 text-sm text-text-muted">
          Build both browser variants from the repository with <code class="rounded bg-surface-base px-1.5 py-0.5 font-mono text-xs text-text-primary">cd app/extension &amp;&amp; npm run build</code>.
        </p>
        <p class="mt-2 text-xs text-text-muted">Supported browsers: Chrome 116+ and Firefox 128+.</p>
        <ol class="mt-4 list-decimal space-y-3 pl-5 text-sm text-text-muted">
          <li>
            <span class="font-medium text-text-primary">Chrome:</span> open <code class="font-mono text-xs text-text-primary">chrome://extensions</code>, enable Developer mode, choose Load unpacked, and select <code class="font-mono text-xs text-text-primary">app/extension/dist/chrome</code>.
          </li>
          <li>
            <span class="font-medium text-text-primary">Firefox:</span> open <code class="font-mono text-xs text-text-primary">about:debugging#/runtime/this-firefox</code>, choose Load Temporary Add-on, and select <code class="font-mono text-xs text-text-primary">app/extension/dist/firefox/manifest.json</code>. Temporary add-ons must be loaded again after Firefox restarts.
          </li>
        </ol>
        <p class="mt-4 text-xs text-text-muted">
          Firefox temporary installation is for local testing. Persistent distribution requires a signed Firefox package.
        </p>
      </section>

      <section class="rounded-xl border border-surface-border bg-surface-elevated/70 p-5 shadow-sm">
        <h2 class="text-sm font-bold text-text-primary">Connect the extension to your account</h2>
        <ol class="mt-3 list-decimal space-y-2 pl-5 text-sm text-text-muted">
          <li>Open the extension popup. On first use, enter your Mensetsu API base origin and select Test connection &amp; continue.</li>
          <li>Use the API origin, not the web app address; enter only the origin, without a path such as <code class="font-mono text-xs text-text-primary">/api/v1</code>. HTTPS is required except for explicitly enabled loopback development.</li>
          <li>Approve the browser prompt to let the extension contact that API. After its health check succeeds, the popup shows the sign-in form.</li>
          <li>Sign in with your Mensetsu account. Changing the saved API origin signs the extension out.</li>
        </ol>
        <p class="mt-4 text-sm text-text-muted">
          The extension never sends email or clicks a final application submit button. It fills forms only on verified supported pages after your approval; all other website applications stay manual.
        </p>
      </section>
      <section class="rounded-xl border border-surface-border bg-surface-elevated/70 p-5 shadow-sm">
        <h2 class="text-sm font-bold text-text-primary">Discover and apply to jobs</h2>
        <ol class="mt-3 list-decimal space-y-2 pl-5 text-sm text-text-muted">
          <li>Sign in to LinkedIn or TopCV in your browser and open a job-search results page. Browser policy prevents the extension from reliably opening its toolbar popup automatically, so click its icon to open Discover and scan visible jobs. Use the site’s normal scrolling or pagination and scan again for more; no background crawl runs. Enable LinkedIn page detection in extension settings before scanning LinkedIn.</li>
          <li>Matching compares your Mensetsu target role, required and excluded keywords, and target locations using accent-insensitive text matching. Add Vietnamese and English variants to your profile when both appear in job listings. Review the captured fields; open the job detail and scan again if the description is incomplete. Save jobs one at a time; missing details are never invented.</li>
          <li>Open a saved job in Apply mode and select a completed PDF resume from Mensetsu. If a visible employer email was captured, review the draft, attach the resume in your mail client, and send it yourself. The extension does not send email or attach files to email.</li>
          <li>Without a usable employer email, open the job’s application page and complete it yourself unless the extension identifies a verified supported form. Unsupported or login-gated forms are not automated.</li>
          <li>After sending the email or submitting the website form, confirm the outcome in the extension. The extension records the confirmed application and queues interview preparation; queued means the work is waiting to be delivered, not that the employer received anything else.</li>
          <li>If you scan the same job again with changed details before its application is final, a prepared, approved, or needs-review application returns to Needs review. Approve the revised facts before preparing again. Submitting, submitted, and unknown applications keep their original facts.</li>
        </ol>
        <p class="mt-4 text-sm text-text-muted">
          Submitted/queued means you confirmed that you submitted the employer application. Processing, Ready, and Failed describe the interview-preparation workspace, not the employer's response. No state sends employer email or clicks the employer's final submit action.
        </p>
      </section>
    </div>

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
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold uppercase text-text-primary">{{ p.name }}</span>
              <span class="rounded px-1.5 py-0.5 text-[10px] font-mono uppercase"
                :class="p.active_key_source === 'user' ? 'bg-primary/10 text-primary' : 'bg-surface-elevated text-text-muted'"
              >
                {{ p.active_key_source }}
              </span>
            </div>
            <p class="mt-1 text-xs text-text-muted truncate">{{ p.base_url || 'Default cloud endpoint' }}</p>
            <div class="mt-3 flex items-center justify-between border-t border-surface-border/50 pt-2 text-xs">
              <button type="button" @click="selectedProvider = p.provider_key" class="text-primary hover:underline font-medium">Configure</button>
              <button v-if="p.is_user_override" type="button" @click="handleClearProviderKey(p.provider_key)" class="text-danger hover:underline">Revert to default</button>
            </div>
          </div>
        </div>

        <div class="rounded-xl border border-surface-border bg-surface-elevated/70 p-5 shadow-sm space-y-4">
          <h3 class="text-sm font-bold text-text-primary uppercase tracking-wide">Configure {{ selectedProvider }}</h3>
          <div v-if="providerFeedback" class="p-3 bg-surface-base text-text-primary text-xs rounded-lg border border-surface-border">
            {{ providerFeedback }}
          </div>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div>
              <label class="block text-xs font-semibold text-text-muted mb-1">API Key</label>
              <input v-model="providerApiKey" type="password" placeholder="Enter key (leave blank to keep current)" class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-primary" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-text-muted mb-1">Base URL (Optional)</label>
              <input v-model="providerBaseUrl" type="url" placeholder="https://api.openai.com/v1" class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary" />
            </div>
          </div>
          <div class="flex justify-end">
            <AppButton size="sm" :loading="isSavingProvider" @click="handleSaveProviderKey">Save credentials</AppButton>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'routing'" class="space-y-6">
      <div class="space-y-5">
        <div class="flex justify-between items-center border-b border-surface-border pb-4">
          <div>
            <h2 class="text-base font-semibold text-text-primary">Pipeline model routing matrix</h2>
            <p class="mt-1 text-sm text-text-muted">Map tasks to specialized models across research and extraction.</p>
          </div>
          <AppButton size="sm" :loading="isSavingMatrix" @click="handleSaveRoutingMatrix">Apply matrix</AppButton>
        </div>

        <div v-if="matrixFeedback" class="p-3 bg-surface-base text-text-primary text-xs rounded-lg border border-surface-border">
          {{ matrixFeedback }}
        </div>

        <div class="grid grid-cols-1 gap-4">
          <div v-for="slot in [
            { id: 'extraction', label: 'CV & Document Extraction', desc: 'Parses structure from uploaded resumes and job descriptions' },
            { id: 'research', label: 'Company & Culture Web Research', desc: 'Gathers public facts, citations, and architecture notes' },
            { id: 'matching', label: 'Gap & Match Analysis', desc: 'Computes multi-dimensional alignment and role fit' },
            { id: 'qa', label: 'Lesson & Interview Question Synthesis', desc: 'Produces learning units and interview question dossiers' },
            { id: 'reasoning', label: 'Orchestration & Verification', desc: 'Validates workflow logic and ensures deterministic execution' },
          ]" :key="slot.id" class="rounded-xl border border-surface-border bg-surface-elevated/70 p-4 shadow-sm">
            <div class="flex justify-between items-start mb-3">
              <div>
                <h3 class="text-xs font-bold text-text-primary uppercase tracking-wide">{{ slot.label }}</h3>
                <p class="text-xs text-text-muted">{{ slot.desc }}</p>
              </div>
            </div>
            <div v-if="matrixState[slot.id]" class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t border-surface-border/50">
              <div>
                <label class="block text-[11px] font-semibold text-text-muted mb-1">Primary model</label>
                <div class="grid grid-cols-2 gap-2">
                  <input
                    v-model="matrixState[slot.id].primary.provider_key"
                    type="text"
                    placeholder="Provider"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary"
                  />
                  <input
                    v-model="matrixState[slot.id].primary.model"
                    type="text"
                    placeholder="Model ID"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary"
                  />
                </div>
              </div>
              <div>
                <label class="block text-[11px] font-semibold text-text-muted mb-1">Fallback model (optional)</label>
                <div v-if="matrixState[slot.id].fallback" class="grid grid-cols-2 gap-2">
                  <input
                    v-model="matrixState[slot.id].fallback!.provider_key"
                    type="text"
                    placeholder="Provider"
                    class="bg-surface-elevated border border-surface-border rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary"
                  />
                  <input
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

    <!-- Security & 2FA Tab -->
    <div v-if="activeTab === 'security'" class="space-y-6">
      <!-- Connected Accounts (OAuth / OIDC) -->
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm space-y-4">
        <div class="border-b border-surface-border pb-3">
          <h2 class="text-sm font-bold text-text-primary">{{ $t('oauth.connectedAccounts') }}</h2>
          <p class="text-xs text-text-muted">{{ $t('oauth.connectedAccountsDesc') }}</p>
        </div>

        <div v-if="oauthFeedback" class="p-3 bg-success/10 text-success text-xs rounded-lg border border-success/20">
          {{ oauthFeedback }}
        </div>

        <div v-if="oauthError" class="p-3 bg-danger/10 text-danger text-xs rounded-lg border border-danger/20">
          {{ oauthError }}
        </div>

        <div v-if="publicOAuthProviders && publicOAuthProviders.length > 0" class="space-y-3">
          <div
            v-for="prov in publicOAuthProviders"
            :key="prov.provider_key"
            class="p-3 bg-surface-base border border-surface-border rounded-lg flex items-center justify-between"
          >
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-full bg-surface-elevated flex items-center justify-center border border-surface-border shrink-0">
                <svg v-if="prov.provider_type === 'google'" class="w-4 h-4" viewBox="0 0 24 24">
                  <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                  <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                  <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                  <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
                </svg>
                <svg v-else-if="prov.provider_type === 'github'" class="w-4 h-4 fill-current text-text-primary" viewBox="0 0 24 24">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
                </svg>
                <svg v-else-if="prov.provider_type === 'facebook'" class="w-4 h-4 fill-[#1877F2]" viewBox="0 0 24 24">
                  <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/>
                </svg>
                <svg v-else class="w-4 h-4 text-text-muted stroke-current fill-none" viewBox="0 0 24 24" stroke-width="2">
                  <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                  <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                </svg>
              </div>

              <div>
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold text-text-primary">{{ prov.name }}</span>
                  <span
                    v-if="getLinkedIdentity(prov.provider_key)"
                    class="px-1.5 py-0.2 text-[10px] font-mono bg-success/15 text-success rounded"
                  >
                    {{ $t('oauth.connected') }}
                  </span>
                </div>
                <p v-if="getLinkedIdentity(prov.provider_key)?.email" class="text-[11px] text-text-muted font-mono">
                  {{ getLinkedIdentity(prov.provider_key)?.email }}
                </p>
              </div>
            </div>

            <div>
              <button
                v-if="getLinkedIdentity(prov.provider_key)"
                type="button"
                @click="handleUnlinkOAuth(getLinkedIdentity(prov.provider_key)!)"
                :disabled="isUnlinkingProvider === prov.provider_key"
                class="px-3 py-1 text-xs text-danger hover:bg-danger/10 border border-danger/30 rounded-lg transition disabled:opacity-50"
              >
                {{ isUnlinkingProvider === prov.provider_key ? 'Disconnecting...' : $t('oauth.disconnect') }}
              </button>
              <button
                v-else
                type="button"
                @click="handleLinkOAuth(prov.provider_key)"
                :disabled="isLinkingProvider === prov.provider_key"
                class="px-3 py-1 text-xs font-medium bg-primary text-white hover:bg-primary-hover rounded-lg transition disabled:opacity-50"
              >
                {{ isLinkingProvider === prov.provider_key ? 'Connecting...' : $t('oauth.connect') }}
              </button>
            </div>
          </div>
        </div>
        <div v-else class="text-xs text-text-muted">
          {{ $t('oauth.noProvidersConfigured') }}
        </div>
      </div>

      <!-- Two-Factor Authentication (TOTP) -->
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm space-y-4">
        <h2 class="text-sm font-bold text-text-primary">Two-Factor Authentication (TOTP)</h2>
        <p class="text-xs text-text-muted">
          Protect your account with RFC 6238 compliant hardware or software authenticators.
        </p>

        <div v-if="twoFaError" class="p-3 bg-danger/10 text-danger text-xs rounded-lg border border-danger/20">
          {{ twoFaError }}
        </div>

        <div v-if="user?.totp_enabled" class="p-4 bg-surface-base border border-surface-border rounded-lg space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-emerald-600 dark:text-emerald-400">2FA is currently active on your account.</span>
            <button
              v-if="!isDisabling2fa"
              type="button"
              @click="isDisabling2fa = true"
              class="px-3 py-1 bg-danger/90 hover:bg-danger text-white rounded text-xs font-medium transition"
            >
              Disable 2FA
            </button>
          </div>
          <div v-if="isDisabling2fa" class="pt-3 border-t border-surface-border/60 space-y-2">
            <p class="text-xs text-text-muted">{{ $t('oauth.disable2fa.description') }}</p>
            <div class="flex gap-2">
              <input
                v-model="totpDisableCode"
                type="text"
                maxlength="8"
                :placeholder="$t('oauth.disable2fa.codePlaceholder')"
                class="bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-danger"
              />
              <button
                type="button"
                @click="handleDisable2fa"
                :disabled="isSubmittingDisable"
                class="px-3 py-1.5 bg-danger text-white rounded-lg text-xs font-semibold hover:bg-danger-hover disabled:opacity-50 transition"
              >
                {{ isSubmittingDisable ? 'Disabling...' : $t('oauth.disable2fa.confirmButton') }}
              </button>
              <button
                type="button"
                @click="isDisabling2fa = false; totpDisableCode = ''; twoFaError = ''"
                class="px-3 py-1.5 text-xs text-text-muted hover:text-text-primary"
              >
                {{ $t('oauth.disable2fa.cancelButton') }}
              </button>
            </div>
          </div>
        </div>

        <div v-else-if="totpSetupData" class="space-y-4 p-4 border border-surface-border rounded-lg">
          <div class="text-xs text-text-muted">Scan QR code with your authenticator app:</div>
          <img
            :src="totpSetupData.qr_code_data_url"
            alt="Authenticator app QR code"
            class="w-48 h-48 bg-white p-2 rounded"
          />
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

    <div v-if="activeTab === 'usage'" class="space-y-6">
      <TokenUsageView />
    </div>
</main>
</div>

</div>
</template>
