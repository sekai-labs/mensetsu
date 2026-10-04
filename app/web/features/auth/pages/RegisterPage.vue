<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
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

const router = useRouter()
const { register, loading } = useAuthService()

const displayName = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const errorMessage = ref('')

const isFormValid = computed(() => {
  return (
    displayName.value.trim().length > 0 &&
    isValidEmail(email.value) &&
    isValidPassword(password.value, 8) &&
    password.value === confirmPassword.value
  )
})

const handleRegister = async () => {
  errorMessage.value = ''
  if (password.value !== confirmPassword.value) {
    errorMessage.value = 'Passwords do not match.'
    return
  }
  if (!isValidPassword(password.value, 8)) {
    errorMessage.value = 'Password must be at least 8 characters long.'
    return
  }

  try {
    await register({
      email: email.value.trim(),
      password: password.value,
      displayName: displayName.value.trim(),
    })
    await router.push('/dashboard')
  } catch (err: unknown) {
    const error = err as ApiErrorPayload
    if (error.data?.message) {
      errorMessage.value = error.data.message
    } else if (error.status === 409) {
      errorMessage.value = 'An account with this email already exists.'
    } else {
      errorMessage.value = error.message || 'Failed to register account. Please check your information and try again.'
    }
  }
}
</script>

<template>
  <AuthCard
    title="Create your Mensetsu account"
    subtitle="Start building evidence-backed technical interview preparations"
  >
    <form class="space-y-4" @submit.prevent="handleRegister">
      <div
        v-if="errorMessage"
        role="alert"
        class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-400"
      >
        {{ errorMessage }}
      </div>

      <div>
        <label for="name" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          Full Name
        </label>
        <div class="mt-1">
          <input
            id="name"
            v-model="displayName"
            name="name"
            type="text"
            autocomplete="name"
            required
            placeholder="Jane Doe"
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-xs text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div>
        <label for="email" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          Work Email
        </label>
        <div class="mt-1">
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

      <div>
        <label for="password" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          Password <span class="text-slate-400 font-normal">(min 8 characters)</span>
        </label>
        <div class="mt-1">
          <input
            id="password"
            v-model="password"
            name="password"
            type="password"
            autocomplete="new-password"
            required
            placeholder="••••••••"
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-xs text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div>
        <label for="confirmPassword" class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
          Confirm Password
        </label>
        <div class="mt-1">
          <input
            id="confirmPassword"
            v-model="confirmPassword"
            name="confirmPassword"
            type="password"
            autocomplete="new-password"
            required
            placeholder="••••••••"
            class="block w-full rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-xs text-slate-900 dark:text-white placeholder-slate-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>

      <div class="pt-2">
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
          <span>{{ loading ? 'Creating account...' : 'Create account' }}</span>
        </button>
      </div>
    </form>

    <template #footer>
      <p class="text-xs text-slate-500 dark:text-slate-400">
        Already have an account?
        <NuxtLink to="/login" class="font-semibold text-indigo-600 dark:text-indigo-400 hover:underline">
          Sign in
        </NuxtLink>
      </p>
    </template>
  </AuthCard>
</template>
