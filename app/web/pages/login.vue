<script setup lang="ts">
import { useAuthService } from '~/features/auth/services/auth.service'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const route = useRoute()
const router = useRouter()
const { login, login2fa } = useAuthService()

const email = ref('')
const password = ref('')
const twoFaCode = ref('')
const tempToken = ref<string | null>(null)
const requires2fa = ref(false)
const isLoading = ref(false)
const errorMessage = ref('')

const handleLogin = async () => {
  errorMessage.value = ''
  isLoading.value = true
  try {
    if (requires2fa.value && tempToken.value) {
      await login2fa({
        temp_token: tempToken.value,
        code: twoFaCode.value.trim(),
      })
      const redirect = (route.query.redirect as string) || '/dashboard'
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
        const redirect = (route.query.redirect as string) || '/dashboard'
        await router.push(redirect)
      }
    }
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      errorMessage.value = err.message
    } else {
      errorMessage.value = 'Failed to sign in. Please verify your credentials.'
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
          Sign In
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          Access your technical interview preparation workspaces
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
            label="Email Address"
            type="email"
            placeholder="engineer@company.com"
            required
          />

          <AppInput
            id="password"
            v-model="password"
            label="Password"
            type="password"
            placeholder="••••••••••••"
            required
          />
        </template>

        <template v-else>
          <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300">
            Two-Factor Authentication is enabled on this account. Enter the 6-digit code from your authenticator app.
          </div>

          <AppInput
            id="2fa-code"
            v-model="twoFaCode"
            label="Authentication Code"
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
          {{ requires2fa ? 'Verify & Sign In' : 'Sign In' }}
        </AppButton>
      </form>

      <div class="pt-4 border-t border-slate-200 dark:border-slate-800 text-center text-xs text-slate-500">
        Don't have an account?
        <NuxtLink to="/register" class="text-indigo-600 dark:text-indigo-400 hover:underline font-semibold ml-1">
          Register here
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
