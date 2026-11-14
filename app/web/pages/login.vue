<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthService } from '~/features/auth/services/auth.service'
import { sanitizeRedirectPath } from '~/features/auth/utils/auth.util'
import type { OAuthProviderSummaryDto } from '~/features/auth/models/auth.model'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const { t, te } = useI18n()
const route = useRoute()
const router = useRouter()
const { login, login2fa } = useAuthService()

const { data: oauthProviders } = await useAsyncData('public-oauth-providers', async () => {
  try {
    return await $fetch<OAuthProviderSummaryDto[]>('/api/auth/oauth/providers')
  } catch {
    return []
  }
})

const email = ref('')
const password = ref('')
const twoFaCode = ref('')
const tempToken = ref<string | null>(null)
const requires2fa = ref(false)
const isLoading = ref(false)
const errorMessage = ref('')

onMounted(() => {
  if (route.query.oauth_error) {
    const errCode = String(route.query.oauth_error)
    const errDesc = route.query.error_description ? String(route.query.error_description) : ''
    if (te(`oauth.errors.${errCode}`)) {
      errorMessage.value = t(`oauth.errors.${errCode}`)
    } else {
      errorMessage.value = errDesc || t('oauth.errors.oauth_failed')
    }
  } else if (route.query.error) {
    errorMessage.value = String(route.query.error_description || route.query.error)
  }

  if (route.query.requires_2fa === 'true' || route.query.requires_2fa === '1') {
    requires2fa.value = true
    if (route.query.temp_token) {
      tempToken.value = String(route.query.temp_token)
    }
  }
})

const handleLogin = async () => {
  errorMessage.value = ''
  isLoading.value = true
  try {
    if (requires2fa.value) {
      await login2fa({
        temp_token: tempToken.value || undefined,
        code: twoFaCode.value.trim(),
      })
      const redirect = sanitizeRedirectPath(route.query.redirect as string)
      await router.push(redirect)
    } else {
      const res = await login({
        email: email.value.trim(),
        password: password.value,
      })
      if (res.requires_2fa && res.temp_token) {
        requires2fa.value = true
        tempToken.value = res.temp_token
      } else {
        const redirect = sanitizeRedirectPath(route.query.redirect as string)
        await router.push(redirect)
      }
    }
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      errorMessage.value = err.message
    } else {
      errorMessage.value = t('auth.loginFailed')
    }
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="py-12 sm:py-16 max-w-sm mx-auto">
    <div class="space-y-6">
      <div class="space-y-1.5 text-center">
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('auth.signInTitle') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          {{ $t('auth.signInSubtitle') }}
        </p>
      </div>

      <div
        v-if="errorMessage"
        class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300"
        role="alert"
      >
        {{ errorMessage }}
      </div>

      <form class="space-y-4" @submit.prevent="handleLogin">
        <template v-if="!requires2fa">
          <AppInput
            id="email"
            v-model="email"
            :label="$t('auth.email')"
            type="email"
            placeholder="engineer@company.com"
            required
          />

          <AppInput
            id="password"
            v-model="password"
            :label="$t('auth.password')"
            type="password"
            placeholder="••••••••••••"
            required
          />
        </template>

        <template v-else>
          <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300">
            {{ $t('auth.twoFaPrompt') }}
          </div>

          <AppInput
            id="2fa-code"
            v-model="twoFaCode"
            :label="$t('auth.twoFaCode')"
            type="text"
            placeholder="123456"
            required
          />
        </template>

        <AppButton
          type="submit"
          class="w-full"
          size="md"
          :loading="isLoading"
        >
          {{ requires2fa ? $t('auth.verifyAndSignIn') : $t('auth.signInButton') }}
        </AppButton>
      </form>

      <!-- OAuth Single Sign-On Provider Buttons -->
      <div v-if="!requires2fa && oauthProviders && oauthProviders.length > 0" class="space-y-3">
        <div class="relative flex items-center justify-center">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-slate-200 dark:border-slate-800" />
          </div>
          <span class="relative bg-white dark:bg-slate-900 px-2 text-[11px] uppercase tracking-wider text-slate-500">
            {{ $t('oauth.orContinueWith') }}
          </span>
        </div>

        <div class="grid gap-2">
          <a
            v-for="prov in oauthProviders"
            :key="prov.provider_key"
            :href="`/api/auth/oauth/${encodeURIComponent(prov.provider_key)}?redirect=${encodeURIComponent(sanitizeRedirectPath(route.query.redirect as string))}`"
            class="flex h-10 w-full items-center justify-center gap-2.5 rounded-md border border-slate-200 bg-white px-4 text-xs font-medium text-slate-700 shadow-xs hover:bg-slate-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-750 transition-colors"
          >
            <!-- Google icon -->
            <svg v-if="prov.provider_type === 'google'" class="h-4 w-4 shrink-0" viewBox="0 0 24 24" aria-hidden="true">
              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
            </svg>
            <!-- GitHub icon -->
            <svg v-else-if="prov.provider_type === 'github'" class="h-4 w-4 shrink-0 fill-current" viewBox="0 0 24 24" aria-hidden="true">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
            </svg>
            <!-- Facebook icon -->
            <svg v-else-if="prov.provider_type === 'facebook'" class="h-4 w-4 shrink-0 fill-[#1877F2]" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/>
            </svg>
            <!-- Custom / OIDC shield icon -->
            <svg v-else class="h-4 w-4 shrink-0 stroke-current fill-none" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
            <span>{{ $t('oauth.signInWith', { name: prov.name }) }}</span>
          </a>
        </div>
      </div>

      <div class="pt-4 border-t border-slate-200 dark:border-slate-800 text-center text-xs text-slate-500">
        {{ $t('auth.noAccount') }}
        <NuxtLink to="/register" class="text-indigo-600 dark:text-indigo-400 hover:underline font-semibold ml-1">
          {{ $t('auth.registerHere') }}
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
