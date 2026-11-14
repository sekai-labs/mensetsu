<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApiClient } from '~/services/api.client'
import type {
  OAuthProviderAdminDto,
  OAuthProviderUpsertRequestDto,
} from '~/features/auth/models/auth.model'

const { t } = useI18n()
const { request } = useApiClient()

const { data: providers, refresh: refreshProviders, status } = await useAsyncData(
  'admin-oauth-providers',
  async () => {
    return await request<AdminOAuthProviderDto[]>('/api/v1/admin/oauth/providers')
  }
)

const isLoading = computed(() => status.value === 'pending')

// Modal state
const isModalOpen = ref(false)
const isEditMode = ref(false)
const modalError = ref('')
const isSubmitting = ref(false)
const showManualEndpoints = ref(false)

const form = ref<OAuthProviderUpsertRequestDto & { provider_key: string; scopesText: string }>({
  provider_key: '',
  name: '',
  provider_type: 'oidc',
  issuer: '',
  client_id: '',
  client_secret: '',
  redirect_uri: '',
  scopesText: '',
  authorization_endpoint: '',
  token_endpoint: '',
  userinfo_endpoint: '',
  email_endpoint: '',
  jwks_uri: '',
  subject_claim: '',
  email_claim: '',
  name_claim: '',
  email_verified_claim: '',
  is_enabled: true,
})

const editingHasSecret = ref(false)

const resetForm = () => {
  form.value = {
    provider_key: '',
    name: '',
    provider_type: 'oidc',
    issuer: '',
    client_id: '',
    client_secret: '',
    redirect_uri: '',
    scopesText: '',
    authorization_endpoint: '',
    token_endpoint: '',
    userinfo_endpoint: '',
    email_endpoint: '',
    jwks_uri: '',
    subject_claim: '',
    email_claim: '',
    name_claim: '',
    email_verified_claim: '',
    is_enabled: true,
  }
  editingHasSecret.value = false
  modalError.value = ''
  showManualEndpoints.value = false
}

const openAddModal = () => {
  resetForm()
  isEditMode.value = false
  isModalOpen.value = true
}

const openEditModal = (provider: OAuthProviderAdminDto) => {
  resetForm()
  isEditMode.value = true
  editingHasSecret.value = provider.has_client_secret
  form.value = {
    provider_key: provider.provider_key,
    name: provider.name,
    provider_type: provider.provider_type,
    issuer: provider.issuer || '',
    client_id: provider.client_id,
    client_secret: '',
    redirect_uri: provider.redirect_uri,
    scopesText: provider.scopes.join(', '),
    authorization_endpoint: provider.authorization_endpoint || '',
    token_endpoint: provider.token_endpoint || '',
    userinfo_endpoint: provider.userinfo_endpoint || '',
    email_endpoint: provider.email_endpoint || '',
    jwks_uri: provider.jwks_uri || '',
    subject_claim: provider.subject_claim || '',
    email_claim: provider.email_claim || '',
    name_claim: provider.name_claim || '',
    email_verified_claim: provider.email_verified_claim || '',
    is_enabled: provider.is_enabled,
  }
  if (
    provider.provider_type === 'oauth2' ||
    provider.authorization_endpoint ||
    provider.token_endpoint ||
    provider.userinfo_endpoint ||
    provider.jwks_uri
  ) {
    showManualEndpoints.value = true
  }
  isModalOpen.value = true
}

const handleSaveProvider = async () => {
  const providerKey = form.value.provider_key.trim().toLowerCase()
  if (!providerKey) {
    modalError.value = 'Provider slug key is required.'
    return
  }
  if (!form.value.client_id?.trim()) {
    modalError.value = 'Client ID is required.'
    return
  }
  if (!form.value.redirect_uri?.trim()) {
    modalError.value = 'OAuth callback URL is required.'
    return
  }
  if (!isEditMode.value && !form.value.client_secret?.trim()) {
    modalError.value = 'Client Secret is required for new providers.'
    return
  }

  isSubmitting.value = true
  modalError.value = ''

  const scopes = form.value.scopesText
    ? form.value.scopesText.split(',').map((scope) => scope.trim()).filter(Boolean)
    : undefined
  const optionalText = (value?: string | null) => value?.trim() || undefined

  const payload: OAuthProviderUpsertRequestDto = {
    name: optionalText(form.value.name) || providerKey,
    provider_type: form.value.provider_type || undefined,
    client_id: form.value.client_id?.trim() ? form.value.client_id : undefined,
    redirect_uri: optionalText(form.value.redirect_uri),
    scopes,
    issuer: optionalText(form.value.issuer),
    authorization_endpoint: optionalText(form.value.authorization_endpoint),
    token_endpoint: optionalText(form.value.token_endpoint),
    userinfo_endpoint: optionalText(form.value.userinfo_endpoint),
    email_endpoint: optionalText(form.value.email_endpoint),
    jwks_uri: optionalText(form.value.jwks_uri),
    subject_claim: optionalText(form.value.subject_claim),
    email_claim: optionalText(form.value.email_claim),
    name_claim: optionalText(form.value.name_claim),
    email_verified_claim: optionalText(form.value.email_verified_claim),
    is_enabled: form.value.is_enabled,
  }

  if (form.value.client_secret?.trim()) {
    payload.client_secret = form.value.client_secret
  }

  try {
    await request(
      `/api/v1/admin/oauth/providers/${encodeURIComponent(providerKey)}`,
      {
        method: 'PUT',
        body: payload,
      }
    )
    isModalOpen.value = false
    await refreshProviders()
  } catch (err: unknown) {
    const error = err as { data?: { error?: { message?: string } }; message?: string }
    modalError.value = error?.data?.error?.message || error?.message || 'Failed to save OAuth provider.'
  } finally {
    isSubmitting.value = false
  }
}

const toggleProviderStatus = async (provider: OAuthProviderAdminDto) => {
  try {
    await request(
      `/api/v1/admin/oauth/providers/${encodeURIComponent(provider.provider_key)}`,
      {
        method: 'PUT',
        body: { is_enabled: !provider.is_enabled },
      }
    )
    await refreshProviders()
  } catch (err: unknown) {
    console.error('Failed to toggle provider status', err)
  }
}

const deleteProviderKey = ref<string | null>(null)
const isDeleting = ref(false)

const confirmDelete = (key: string) => {
  deleteProviderKey.value = key
}

const handleDelete = async () => {
  if (!deleteProviderKey.value) return
  isDeleting.value = true
  try {
    await request(
      `/api/v1/admin/oauth/providers/${encodeURIComponent(deleteProviderKey.value)}`,
      {
        method: 'DELETE',
      }
    )
    deleteProviderKey.value = null
    await refreshProviders()
  } catch (err: any) {
    console.error('Failed to delete provider', err)
  } finally {
    isDeleting.value = false
  }
}
</script>

<template>
  <section class="space-y-5 border-b border-surface-border pb-7">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <div>
        <h2 class="text-sm font-bold text-text-primary">
          {{ $t('oauth.admin.title') }}
        </h2>
        <p class="text-xs text-text-muted">
          {{ $t('oauth.admin.subtitle') }}
        </p>
      </div>
      <button
        type="button"
        @click="openAddModal"
        class="px-3.5 py-1.5 text-xs font-medium bg-primary text-white rounded-lg hover:bg-primary-hover transition flex items-center gap-1.5 shrink-0"
      >
        <span>+ {{ $t('oauth.admin.addProvider') }}</span>
      </button>
    </div>

    <!-- Provider Cards Grid -->
    <div v-if="providers && providers.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 pt-2">
      <div
        v-for="p in providers"
        :key="p.provider_key"
        class="p-4 bg-surface-base border border-surface-border rounded-lg flex flex-col justify-between space-y-3"
      >
        <div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-text-primary uppercase tracking-wide">{{ p.name }}</span>
              <span class="px-1.5 py-0.5 text-[10px] font-mono bg-surface-elevated text-text-muted rounded border border-surface-border">
                {{ p.provider_type }}
              </span>
            </div>
            <span
              class="px-1.5 py-0.5 text-[10px] font-mono rounded font-medium"
              :class="p.is_enabled ? 'bg-success/15 text-success' : 'bg-danger/15 text-danger'"
            >
              {{ p.is_enabled ? $t('oauth.admin.enabled') : 'Disabled' }}
            </span>
          </div>

          <p class="text-[11px] text-text-muted mt-2 font-mono truncate">
            Client ID: {{ p.client_id }}
          </p>
          <p v-if="p.issuer" class="text-[11px] text-text-muted font-mono truncate">
            Issuer: {{ p.issuer }}
          </p>
          <p class="text-[11px] text-text-muted font-mono truncate">
            Callback URI: {{ p.redirect_uri }}
          </p>

          <div class="mt-2 flex items-center gap-2">
            <span
              class="text-[10px] font-mono"
              :class="p.has_client_secret ? 'text-text-muted' : 'text-amber-500'"
            >
              {{ p.has_client_secret ? '● Secret Redacted & Configured' : '○ No Secret' }}
            </span>
          </div>
        </div>

        <div class="pt-2 border-t border-surface-border/50 flex items-center justify-between text-xs">
          <div class="flex items-center gap-3">
            <button
              type="button"
              @click="openEditModal(p)"
              class="text-primary hover:underline font-medium text-[11px]"
            >
              {{ $t('oauth.admin.editProvider') }}
            </button>
            <button
              type="button"
              @click="toggleProviderStatus(p)"
              class="font-medium text-[11px]"
              :class="p.is_enabled ? 'text-text-muted hover:text-text-primary' : 'text-success hover:underline'"
            >
              {{ p.is_enabled ? 'Disable' : 'Enable' }}
            </button>
          </div>
          <button
            type="button"
            @click="confirmDelete(p.provider_key)"
            class="text-danger hover:underline font-medium text-[11px]"
          >
            {{ $t('oauth.admin.deleteProvider') }}
          </button>
        </div>
      </div>
    </div>

    <div
      v-else-if="!isLoading"
      class="p-6 border border-dashed border-surface-border rounded-lg text-center text-xs text-text-muted"
    >
      {{ $t('oauth.noProvidersConfigured') }}
    </div>

    <!-- Upsert Provider Modal -->
    <div
      v-if="isModalOpen"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
    >
      <div class="w-full max-w-lg bg-surface-base border border-surface-border rounded-xl shadow-xl p-5 space-y-4 max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between border-b border-surface-border pb-3">
          <h3 class="text-sm font-bold text-text-primary">
            {{ isEditMode ? $t('oauth.admin.editProvider') : $t('oauth.admin.addProvider') }}
          </h3>
          <button @click="isModalOpen = false" class="text-text-muted hover:text-text-primary text-sm">✕</button>
        </div>

        <div v-if="modalError" class="p-3 bg-danger/10 text-danger text-xs rounded-lg border border-danger/20">
          {{ modalError }}
        </div>

        <div class="space-y-3 text-xs">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.providerKey') }}</label>
              <input
                v-model="form.provider_key"
                type="text"
                :disabled="isEditMode"
                placeholder="google, github, okta-sso"
                class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary font-mono disabled:opacity-60"
              />
            </div>
            <div>
              <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.displayName') }}</label>
              <input
                v-model="form.name"
                type="text"
                placeholder="Google Workspace, Okta Corporate"
                class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
              />
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.providerType') }}</label>
              <select
                v-model="form.provider_type"
                class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
              >
                <option value="google">Google</option>
                <option value="github">GitHub</option>
                <option value="facebook">Facebook</option>
                <option value="oidc">OpenID Connect (OIDC)</option>
                <option value="oauth2">OAuth 2.0</option>
              </select>
            </div>
            <div>
              <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.clientId') }}</label>
              <input
                v-model="form.client_id"
                type="text"
                placeholder="Client ID from provider console"
                class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-primary"
              />
            </div>
          </div>

          <!-- Redacted Secret Field -->
          <div>
            <label class="block font-semibold text-text-muted mb-1">
              {{ $t('oauth.admin.clientSecret') }}
              <span v-if="editingHasSecret" class="text-text-muted font-normal text-[11px] ml-1">
                ({{ $t('oauth.admin.clientSecretRedacted') }})
              </span>
            </label>
            <input
              v-model="form.client_secret"
              type="password"
              :placeholder="editingHasSecret ? $t('oauth.admin.clientSecretPlaceholder') : 'Enter provider client secret'"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-primary"
            />
            <p v-if="editingHasSecret" class="text-[11px] text-text-muted mt-1">
              {{ $t('oauth.admin.clientSecretPlaceholder') }}
            </p>
          </div>

          <div>
            <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.issuerUrl') }}</label>
            <input
              v-model="form.issuer"
              type="url"
              placeholder="https://accounts.google.com or https://auth.company.com"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block font-semibold text-text-muted mb-1">{{ $t('oauth.admin.scopes') }}</label>
            <input
              v-model="form.scopesText"
              type="text"
              placeholder="openid, email, profile"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Manual Endpoints Toggle -->
          <div class="pt-2 border-t border-surface-border/50">
            <button
              type="button"
              @click="showManualEndpoints = !showManualEndpoints"
              class="text-xs text-text-muted hover:text-text-primary flex items-center gap-1 font-semibold"
            >
              <span>{{ showManualEndpoints ? '▾' : '▸' }}</span>
              <span>{{ $t('oauth.admin.manualEndpoints') }}</span>
            </button>

            <div v-if="showManualEndpoints" class="space-y-2.5 pt-2">
              <div>
                <label class="block text-[11px] text-text-muted mb-0.5">{{ $t('oauth.admin.authEndpoint') }}</label>
                <input
                  v-model="form.authorization_endpoint"
                  type="url"
                  placeholder="https://auth.company.com/oauth/authorize"
                  class="w-full bg-surface-elevated border border-surface-border rounded px-2.5 py-1 text-xs text-text-primary"
                />
              </div>
              <div>
                <label class="block text-[11px] text-text-muted mb-0.5">{{ $t('oauth.admin.tokenEndpoint') }}</label>
                <input
                  v-model="form.token_endpoint"
                  type="url"
                  placeholder="https://auth.company.com/oauth/token"
                  class="w-full bg-surface-elevated border border-surface-border rounded px-2.5 py-1 text-xs text-text-primary"
                />
              </div>
              <div>
                <label class="block text-[11px] text-text-muted mb-0.5">{{ $t('oauth.admin.userinfoEndpoint') }}</label>
                <input
                  v-model="form.userinfo_endpoint"
                  type="url"
                  placeholder="https://auth.company.com/oauth/userinfo"
                  class="w-full bg-surface-elevated border border-surface-border rounded px-2.5 py-1 text-xs text-text-primary"
                />
              </div>
              <div>
                <label class="block text-[11px] text-text-muted mb-0.5">{{ $t('oauth.admin.jwksUri') }}</label>
                <input
                  v-model="form.jwks_uri"
                  type="url"
                  placeholder="https://auth.company.com/.well-known/jwks.json"
                  class="w-full bg-surface-elevated border border-surface-border rounded px-2.5 py-1 text-xs text-text-primary"
                />
              </div>
            </div>
          </div>

          <div class="space-y-2 border-t border-surface-border/50 pt-3">
            <p class="font-semibold text-text-muted">User info claim mapping</p>
            <p class="text-[11px] text-text-muted">Leave blank to use standard claims such as sub, email, name, and email_verified.</p>
            <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <input v-model="form.subject_claim" type="text" placeholder="Subject claim (sub)" class="rounded border border-surface-border bg-surface-elevated px-2.5 py-1 text-xs text-text-primary" />
              <input v-model="form.email_claim" type="text" placeholder="Email claim (email)" class="rounded border border-surface-border bg-surface-elevated px-2.5 py-1 text-xs text-text-primary" />
              <input v-model="form.name_claim" type="text" placeholder="Display name claim (name)" class="rounded border border-surface-border bg-surface-elevated px-2.5 py-1 text-xs text-text-primary" />
              <input v-model="form.email_verified_claim" type="text" placeholder="Verified-email claim (email_verified)" class="rounded border border-surface-border bg-surface-elevated px-2.5 py-1 text-xs text-text-primary" />
            </div>
          </div>
              <div>
                <label class="block text-[11px] text-text-muted mb-0.5">Email endpoint (optional)</label>
                <input
                  v-model="form.email_endpoint"
                  type="url"
                  placeholder="https://auth.company.com/user/emails"
                  class="w-full bg-surface-elevated border border-surface-border rounded px-2.5 py-1 text-xs text-text-primary"
                />
              </div>
          <div>
            <label class="block font-semibold text-text-muted mb-1">OAuth callback URL (register with provider)</label>
            <input
              v-model="form.redirect_uri"
              type="url"
              required
              placeholder="https://api.example.com/api/v1/auth/oauth/callback"
              class="w-full bg-surface-elevated border border-surface-border rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <!-- Flags -->
          <div class="pt-2 border-t border-surface-border/50 flex flex-col sm:flex-row gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input v-model="form.is_enabled" type="checkbox" class="rounded text-primary focus:ring-0" />
              <span class="font-medium text-text-primary">{{ $t('oauth.admin.enabled') }}</span>
            </label>
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-surface-border">
          <button
            type="button"
            @click="isModalOpen = false"
            class="px-3.5 py-1.5 text-xs text-text-muted hover:text-text-primary"
          >
            {{ $t('oauth.admin.cancel') }}
          </button>
          <button
            type="button"
            @click="handleSaveProvider"
            :disabled="isSubmitting"
            class="px-4 py-1.5 text-xs font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50 transition"
          >
            {{ isSubmitting ? 'Saving...' : $t('oauth.admin.save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Confirm Delete Modal -->
    <div
      v-if="deleteProviderKey"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
    >
      <div class="w-full max-w-sm bg-surface-base border border-surface-border rounded-xl shadow-xl p-5 space-y-4">
        <h3 class="text-sm font-bold text-text-primary">{{ $t('oauth.admin.deleteProvider') }}</h3>
        <p class="text-xs text-text-muted">
          {{ $t('oauth.admin.deleteConfirm', { key: deleteProviderKey }) }}
        </p>
        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            type="button"
            @click="deleteProviderKey = null"
            class="px-3 py-1.5 text-xs text-text-muted hover:text-text-primary"
          >
            {{ $t('oauth.admin.cancel') }}
          </button>
          <button
            type="button"
            @click="handleDelete"
            :disabled="isDeleting"
            class="px-4 py-1.5 text-xs font-semibold bg-danger text-white rounded-lg hover:bg-danger/90 disabled:opacity-50 transition"
          >
            {{ isDeleting ? 'Deleting...' : $t('oauth.admin.deleteProvider') }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
