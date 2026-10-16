<script setup lang="ts">
import { useApiClient } from '~/services/api.client'
import { useAuthService } from '~/features/auth/services/auth.service'

definePageMeta({
  layout: 'default',
})

const { user } = useAuthService()
const { request } = useApiClient()
const router = useRouter()

if (user.value && user.value.role !== 'admin') {
  router.push('/dashboard')
}

// 1. Dynamic System Settings
interface SystemConfigMap {
  MAX_CONCURRENT_AGENTS?: number
  MAX_CONCURRENT_AGENTS_PER_RUN?: number
  LLM_MAX_INFLIGHT?: number
  ACTIVE_SEARCH_PROVIDER?: string
  SEARXNG_URL?: string
  TAVILY_API_KEY?: string
  BRAVE_API_KEY?: string
}

const { data: configData, refresh: refreshConfig } = await useAsyncData('admin-sys-config', async () => {
  return await request<SystemConfigMap>('/api/v1/admin/settings')
})

const configForm = ref<SystemConfigMap>({
  MAX_CONCURRENT_AGENTS: 5,
  MAX_CONCURRENT_AGENTS_PER_RUN: 3,
  LLM_MAX_INFLIGHT: 10,
  ACTIVE_SEARCH_PROVIDER: 'searxng',
  SEARXNG_URL: '',
  TAVILY_API_KEY: '',
  BRAVE_API_KEY: '',
})

watch(
  configData,
  (val) => {
    if (val) {
      configForm.value = { ...configForm.value, ...val }
    }
  },
  { immediate: true }
)

const isSavingConfig = ref(false)
const configFeedback = ref('')

const handleSaveConfig = async () => {
  isSavingConfig.value = true
  configFeedback.value = ''
  try {
    await request('/api/v1/admin/settings', {
      method: 'PUT',
      body: configForm.value,
    })
    configFeedback.value = 'Dynamic system settings successfully updated.'
    await refreshConfig()
  } catch (err: any) {
    configFeedback.value = err?.data?.error?.message || err?.message || 'Failed to update settings.'
  } finally {
    isSavingConfig.value = false
  }
}

// 2. Admin Providers Management
interface AdminProvider {
  provider_key: string
  name: string
  base_url?: string
  is_enabled: boolean
  allowed_models: string[]
  has_key: boolean
}

const { data: adminProviders, refresh: refreshProviders } = await useAsyncData('admin-providers', async () => {
  return await request<AdminProvider[]>('/api/v1/admin/providers')
})

const showNewProviderModal = ref(false)
const newProviderKey = ref('')
const newProviderName = ref('')
const newProviderBaseUrl = ref('')
const newProviderApiKey = ref('')
const isAddingProvider = ref(false)
const providerModalError = ref('')

const handleAddProvider = async () => {
  if (!newProviderKey.value.trim()) return
  isAddingProvider.value = true
  providerModalError.value = ''
  try {
    await request('/api/v1/admin/providers', {
      method: 'POST',
      body: {
        provider_key: newProviderKey.value.trim().toLowerCase(),
        name: newProviderName.value.trim() || newProviderKey.value.trim(),
        base_url: newProviderBaseUrl.value.trim() || undefined,
        api_key: newProviderApiKey.value.trim() || undefined,
        is_enabled: true,
      },
    })
    showNewProviderModal.value = false
    newProviderKey.value = ''
    newProviderName.value = ''
    newProviderBaseUrl.value = ''
    newProviderApiKey.value = ''
    await refreshProviders()
  } catch (err: any) {
    providerModalError.value = err?.data?.error?.message || err?.message || 'Failed to add provider.'
  } finally {
    isAddingProvider.value = false
  }
}

const toggleProvider = async (p: AdminProvider) => {
  try {
    await request(`/api/v1/admin/providers/${p.provider_key}`, {
      method: 'PUT',
      body: {
        provider_key: p.provider_key,
        name: p.name,
        base_url: p.base_url,
        is_enabled: !p.is_enabled,
      },
    })
    await refreshProviders()
  } catch (err: any) {
    console.error(err)
  }
}
</script>

<template>
  <div class="max-w-5xl mx-auto py-8 px-4 sm:px-6 space-y-8">
    <div class="border-b border-surface-border pb-5">
      <div class="flex items-center gap-2 mb-1">
        <span class="px-2 py-0.5 text-xs font-mono font-medium bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-slate-700 rounded">
          System Administration
        </span>
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-text-primary">Platform Dynamic Configuration</h1>
      <p class="text-xs text-text-muted mt-1">
        Dynamically tune concurrency limits, LLM in-flight throttles, search providers, and platform gateways.
      </p>
    </div>

    <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm space-y-5">
      <div class="flex justify-between items-center">
        <div>
          <h2 class="text-sm font-bold text-text-primary">Worker Concurrency & Throttle Controls</h2>
          <p class="text-xs text-text-muted">Instant in-flight limits applied globally across research worker processes.</p>
        </div>
        <button
          type="button"
          @click="handleSaveConfig"
          :disabled="isSavingConfig"
          class="px-4 py-1.5 text-xs font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50 transition"
        >
          {{ isSavingConfig ? 'Saving...' : 'Apply System Config' }}
        </button>
      </div>

      <div v-if="configFeedback" class="p-3 rounded-lg text-xs" :class="configFeedback.includes('Failed') ? 'bg-danger/10 text-danger border border-danger/20' : 'bg-success/10 text-success border border-success/20'">
        {{ configFeedback }}
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-5 pt-2">
        <div>
          <label class="block text-xs font-semibold text-text-muted mb-1">Max Concurrent Agents (Global)</label>
          <input
            v-model.number="configForm.MAX_CONCURRENT_AGENTS"
            type="number"
            min="1"
            max="50"
            class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary font-mono"
          />
        </div>

        <div>
          <label class="block text-xs font-semibold text-text-muted mb-1">Max Agents Per Run</label>
          <input
            v-model.number="configForm.MAX_CONCURRENT_AGENTS_PER_RUN"
            type="number"
            min="1"
            max="20"
            class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary font-mono"
          />
        </div>

        <div>
          <label class="block text-xs font-semibold text-text-muted mb-1">LLM Max In-Flight Calls</label>
          <input
            v-model.number="configForm.LLM_MAX_INFLIGHT"
            type="number"
            min="1"
            max="100"
            class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary font-mono"
          />
        </div>
      </div>

      <div class="pt-4 border-t border-surface-border/60 space-y-4">
        <h3 class="text-xs font-bold uppercase tracking-wider text-text-muted">Active Search Engine Backend</h3>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Active Search Provider</label>
            <select
              v-model="configForm.ACTIVE_SEARCH_PROVIDER"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            >
              <option value="searxng">SearXNG (Self-Hosted Private)</option>
              <option value="tavily">Tavily API</option>
              <option value="brave">Brave Search API</option>
            </select>
          </div>

          <div v-if="configForm.ACTIVE_SEARCH_PROVIDER === 'searxng'">
            <label class="block text-xs font-medium text-text-muted mb-1">SearXNG Base URL</label>
            <input
              v-model="configForm.SEARXNG_URL"
              type="url"
              placeholder="http://mensetsu-searxng:8080"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div v-if="configForm.ACTIVE_SEARCH_PROVIDER === 'tavily'">
            <label class="block text-xs font-medium text-text-muted mb-1">Tavily API Key</label>
            <input
              v-model="configForm.TAVILY_API_KEY"
              type="password"
              placeholder="tvly-..."
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary font-mono"
            />
          </div>

          <div v-if="configForm.ACTIVE_SEARCH_PROVIDER === 'brave'">
            <label class="block text-xs font-medium text-text-muted mb-1">Brave Search API Key</label>
            <input
              v-model="configForm.BRAVE_API_KEY"
              type="password"
              placeholder="BSA..."
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary font-mono"
            />
          </div>
        </div>
      </div>
    </div>

    <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm space-y-4">
      <div class="flex justify-between items-center">
        <div>
          <h2 class="text-sm font-bold text-text-primary">Platform Model Gateways & Shared Credentials</h2>
          <p class="text-xs text-text-muted">Global LLM providers shared by users who do not provide their own keys.</p>
        </div>
        <button
          type="button"
          @click="showNewProviderModal = true"
          class="px-3.5 py-1.5 text-xs font-medium bg-primary text-white rounded-lg hover:bg-primary-hover transition flex items-center gap-1.5"
        >
          <span>+ Add Custom Gateway</span>
        </button>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3 pt-2">
        <div
          v-for="p in (adminProviders || [])"
          :key="p.provider_key"
          class="p-4 bg-surface-base border border-surface-border rounded-lg flex flex-col justify-between"
        >
          <div>
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-text-primary uppercase">{{ p.name }}</span>
              <span
                class="px-1.5 py-0.5 text-[10px] font-mono rounded font-medium"
                :class="p.is_enabled ? 'bg-success/15 text-success' : 'bg-danger/15 text-danger'"
              >
                {{ p.is_enabled ? 'Enabled' : 'Disabled' }}
              </span>
            </div>
            <p class="text-[11px] text-text-muted mt-1 truncate font-mono">{{ p.base_url || 'Default Cloud Endpoint' }}</p>
          </div>

          <div class="mt-4 pt-2 border-t border-surface-border/50 flex items-center justify-between">
            <span class="text-[10px] text-text-muted">{{ p.has_key ? '● Key Configured' : '○ No Key' }}</span>
            <button
              type="button"
              @click="toggleProvider(p)"
              class="text-[11px] font-medium"
              :class="p.is_enabled ? 'text-danger hover:underline' : 'text-primary hover:underline'"
            >
              {{ p.is_enabled ? 'Disable' : 'Enable' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showNewProviderModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
      <div class="w-full max-w-md bg-surface-base border border-surface-border rounded-xl shadow-xl p-5 space-y-4">
        <div class="flex items-center justify-between border-b border-surface-border pb-3">
          <h3 class="text-sm font-bold text-text-primary">Add Custom Platform LLM Gateway</h3>
          <button @click="showNewProviderModal = false" class="text-text-muted hover:text-text-primary text-sm">✕</button>
        </div>

        <div v-if="providerModalError" class="p-3 bg-danger/10 text-danger text-xs rounded-lg border border-danger/20">
          {{ providerModalError }}
        </div>

        <div class="space-y-3">
          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Provider Slug Key (e.g. vllm, openrouter)</label>
            <input
              v-model="newProviderKey"
              type="text"
              placeholder="slug-key"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Display Name</label>
            <input
              v-model="newProviderName"
              type="text"
              placeholder="e.g. Local vLLM Cluster"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Base URL</label>
            <input
              v-model="newProviderBaseUrl"
              type="url"
              placeholder="http://vllm-service:8000/v1"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Shared Platform API Key</label>
            <input
              v-model="newProviderApiKey"
              type="password"
              placeholder="sk-..."
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-2 border-t border-surface-border">
          <button
            type="button"
            @click="showNewProviderModal = false"
            class="px-3 py-1.5 text-xs text-text-muted hover:text-text-primary"
          >
            Cancel
          </button>
          <button
            type="button"
            @click="handleAddProvider"
            :disabled="!newProviderKey.trim() || isAddingProvider"
            class="px-4 py-1.5 text-xs font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50 transition"
          >
            {{ isAddingProvider ? 'Adding...' : 'Save Provider' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
