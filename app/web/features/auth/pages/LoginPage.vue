<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import { useAuthService } from '../services/auth.service'
import { isValidEmail, isValidPassword } from '../utils/auth.util'

interface ApiErrorPayload {
  data?: {
    message?: string
  }
  status?: number
  message?: string
}

const route = useRoute()
const router = useRouter()
const { login, login2fa, loading } = useAuthService()

const email = ref('')
const password = ref('')
const errorMessage = ref('')

const requires2fa = ref(false)
const tempToken = ref('')
const totpCode = ref('')

const isFormValid = computed(() => {
  if (requires2fa.value) {
    return totpCode.value.length === 6
  }
  return isValidEmail(email.value) && isValidPassword(password.value, 6)
})

const handleLogin = async () => {
  errorMessage.value = ''
  if (!isFormValid.value) {
    errorMessage.value = requires2fa.value
      ? 'Please enter your 6-digit authenticator code.'
      : 'Please provide a valid email and password (minimum 6 characters).'
    return
  }

  try {
    if (requires2fa.value) {
      await login2fa(tempToken.value, totpCode.value)
      const redirectQuery = route.query.redirect
      const redirect = typeof redirectQuery === 'string' && redirectQuery.startsWith('/') ? redirectQuery : '/dashboard'
      await router.push(redirect)
      return
    }

    const result = await login({
      email: email.value.trim(),
      password: password.value,
    })

    if (result.requires_2fa && result.temp_token) {
      requires2fa.value = true
      tempToken.value = result.temp_token
      return
    }

    const redirectQuery = route.query.redirect
    const redirect = typeof redirectQuery === 'string' && redirectQuery.startsWith('/') ? redirectQuery : '/dashboard'
    await router.push(redirect)
  } catch (err: unknown) {
    const error = err as ApiErrorPayload
    if (error.data?.message) {
      errorMessage.value = error.data.message
    } else if (error.status === 401) {
      errorMessage.value = requires2fa.value ? 'Invalid 2FA code.' : 'Invalid email or password.'
    } else {
      errorMessage.value = error.message || 'Failed to sign in. Please verify your credentials and try again.'
    }
  }
}
</script>

<template>
  <AuthCard
    :title="requires2fa ? 'Two-Factor Authentication' : 'Sign in to Mensetsu'"
    :subtitle="requires2fa ? 'Enter the 6-digit code from your authenticator app' : 'Evidence-backed technical interview intelligence workspace'"
  >
    <form class="space-y-5" @submit.prevent="handleLogin">
      <div
        v-if="errorMessage"
        role="alert"
        class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-400"
      >
        {{ errorMessage }}
      </div>

      <div v-if="!requires2fa">
        <label for="email" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          Work Email
        </label>
        <div class="mt-1.5">
          <input
            id="email"
            v-model="email"
            name="email"
            type="email"
            autocomplete="email"
            required
            placeholder="developer@company.com"
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-xs text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div v-if="!requires2fa">
        <div class="flex items-center justify-between">
          <label for="password" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
            Password
          </label>
        </div>
        <div class="mt-1.5">
          <input
            id="password"
            v-model="password"
            name="password"
            type="password"
            autocomplete="current-password"
            required
            placeholder="••••••••"
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-xs text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div v-else>
        <label for="totp" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          6-digit Security Code
        </label>
        <div class="mt-1.5">
          <input
            id="totp"
            v-model="totpCode"
            type="text"
            maxlength="6"
            autocomplete="one-time-code"
            required
            placeholder="000000"
            autofocus
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2.5 text-center font-mono tracking-widest text-base text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>
      <div>
        <button
          type="submit"
          :disabled="loading || !isFormValid"
          class="w-full flex justify-center items-center py-2 px-4 border border-transparent rounded-md shadow-sm text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition"
        >
          <svg
            v-if="loading"
            class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          <span>{{ loading ? 'Verifying...' : (requires2fa ? 'Verify & Sign In' : 'Sign in') }}</span>
        </button>
        <button
          v-if="requires2fa"
          type="button"
          @click="requires2fa = false; totpCode = ''; errorMessage = ''"
          class="w-full mt-2 py-1.5 text-xs text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition"
        >
          ← Back to Email & Password
        </button>
      </div>
    </form>

    <template #footer>
      <p class="text-xs text-slate-500 dark:text-slate-400">
        Don't have an account?
        <NuxtLink to="/register" class="font-semibold text-indigo-600 dark:text-indigo-400 hover:underline">
          Create one now
        </NuxtLink>
      </p>
    </template>
  </AuthCard>
</template>
