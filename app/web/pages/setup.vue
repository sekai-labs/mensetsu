<script setup lang="ts">
import { useApiClient } from '~/services/api.client'

definePageMeta({
  layout: 'default',
})

const { request } = useApiClient()
const router = useRouter()

const adminName = ref('')
const adminEmail = ref('')
const adminPassword = ref('')
const platformProvider = ref('openai')
const platformApiKey = ref('')
const platformBaseUrl = ref('')

const isSubmitting = ref(false)
const errorMessage = ref('')

const { data: status, refresh: refreshStatus } = await useAsyncData('setup-status', async () => {
  return await request<{ is_initialized: boolean; has_admin: boolean }>('/api/v1/setup/status')
})

if (status.value?.is_initialized) {
  router.push('/login')
}

const handleInitialize = async () => {
  errorMessage.value = ''
  if (!adminName.value.trim() || !adminEmail.value.trim() || adminPassword.value.length < 8) {
    errorMessage.value = 'Please provide admin name, valid email, and at least 8 characters for password.'
    return
  }

  isSubmitting.value = true
  try {
    await request('/api/v1/setup/initialize', {
      method: 'POST',
      body: {
        admin_name: adminName.value.trim(),
        admin_email: adminEmail.value.trim(),
        admin_password: adminPassword.value,
        platform_llm_provider: platformProvider.value,
        platform_llm_api_key: platformApiKey.value.trim() || undefined,
        platform_llm_base_url: platformBaseUrl.value.trim() || undefined,
      },
    })
    router.push('/login')
  } catch (err: any) {
    errorMessage.value = err?.data?.error?.message || err?.message || 'Failed to initialize setup.'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="min-h-[80vh] flex items-center justify-center py-12 px-4 sm:px-6">
    <div class="max-w-md w-full space-y-8 bg-surface-elevated/70 border border-surface-border rounded-2xl p-8 shadow-sm">
      <div class="text-center">
        <div class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-mono font-medium bg-primary/10 text-primary border border-primary/20 rounded-full mb-3">
          <span>First-Run Wizard</span>
        </div>
        <h2 class="text-2xl font-bold tracking-tight text-text-primary">Initialize Mensetsu</h2>
        <p class="text-xs text-text-muted mt-1">Configure root administrator and initial platform model provider.</p>
      </div>

      <div v-if="errorMessage" class="p-3 bg-danger/10 border border-danger/20 text-danger text-xs rounded-lg">
        {{ errorMessage }}
      </div>

      <form @submit.prevent="handleInitialize" class="space-y-5">
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Administrator Full Name</label>
            <input
              v-model="adminName"
              type="text"
              required
              placeholder="e.g. Platform Administrator"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Administrator Email</label>
            <input
              v-model="adminEmail"
              type="email"
              required
              placeholder="admin@mensetsu.internal"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-semibold text-text-muted mb-1">Root Password (min. 8 chars)</label>
            <input
              v-model="adminPassword"
              type="password"
              required
              placeholder="••••••••••••"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        <div class="pt-4 border-t border-surface-border space-y-4">
          <div class="text-xs font-bold uppercase tracking-wider text-text-muted">Platform Shared LLM (Optional)</div>

          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Default Provider</label>
            <select
              v-model="platformProvider"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            >
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="openrouter">OpenRouter</option>
              <option value="gemini">Google Gemini</option>
            </select>
          </div>

          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Platform API Key</label>
            <input
              v-model="platformApiKey"
              type="password"
              placeholder="sk-..."
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Custom Base URL (Optional)</label>
            <input
              v-model="platformBaseUrl"
              type="url"
              placeholder="https://..."
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        <button
          type="submit"
          :disabled="isSubmitting"
          class="w-full py-2.5 rounded-lg bg-primary text-white font-medium text-sm hover:bg-primary-hover disabled:opacity-50 transition flex items-center justify-center gap-2 shadow-sm"
        >
          <span v-if="isSubmitting" class="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"></span>
          <span>{{ isSubmitting ? 'Initializing Platform...' : 'Complete First-Run Setup' }}</span>
        </button>
      </form>
    </div>
  </div>
</template>
