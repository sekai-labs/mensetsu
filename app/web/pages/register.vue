<script setup lang="ts">
import { useAuthService } from '~/features/auth/services/auth.service'
import { isValidEmail, validatePasswordStrength } from '~/features/auth/utils/auth.util'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const router = useRouter()
const { register } = useAuthService()

const name = ref('')
const email = ref('')
const password = ref('')
const isLoading = ref(false)
const errorMessage = ref('')

const handleRegister = async () => {
  errorMessage.value = ''

  if (!isValidEmail(email.value.trim())) {
    errorMessage.value = 'Please provide a valid email address.'
    return
  }

  const strength = validatePasswordStrength(password.value)
  if (!strength.valid) {
    errorMessage.value = 'Password must be at least 8 characters long and contain numbers or special characters.'
    return
  }

  isLoading.value = true
  try {
    await register({
      name: name.value.trim(),
      email: email.value.trim(),
      password: password.value,
    })
    await router.push('/dashboard')
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      errorMessage.value = err.message
    } else {
      errorMessage.value = 'Failed to register account. Please try again.'
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
          Create Account
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          Start structured interview preparation with real evidence
        </p>
      </div>

      <div
        v-if="errorMessage"
        class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300"
        role="alert"
      >
        {{ errorMessage }}
      </div>

      <form class="space-y-4" @submit.prevent="handleRegister">
        <AppInput
          id="name"
          v-model="name"
          label="Full Name"
          type="text"
          placeholder="Ada Lovelace"
          required
        />

        <AppInput
          id="email"
          v-model="email"
          label="Work or Personal Email"
          type="email"
          placeholder="engineer@company.com"
          required
        />

        <AppInput
          id="password"
          v-model="password"
          label="Password (min 8 characters)"
          type="password"
          placeholder="••••••••••••"
          required
        />

        <AppButton
          type="submit"
          class="w-full"
          size="md"
          :loading="isLoading"
        >
          Create Account
        </AppButton>
      </form>

      <div class="pt-4 border-t border-slate-200 dark:border-slate-800 text-center text-xs text-slate-500">
        Already have an account?
        <NuxtLink to="/login" class="text-indigo-600 dark:text-indigo-400 hover:underline font-semibold ml-1">
          Sign in here
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
