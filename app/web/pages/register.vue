<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthService } from '~/features/auth/services/auth.service'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()
const router = useRouter()
const { register } = useAuthService()

const name = ref('')
const email = ref('')
const password = ref('')
const isLoading = ref(false)
const errorMessage = ref('')

const handleRegister = async () => {
  errorMessage.value = ''
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
      errorMessage.value = 'Failed to register account. Please check your information.'
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
          {{ $t('auth.createAccountTitle') }}
        </h1>
        <p class="text-xs text-slate-500 dark:text-slate-400">
          {{ $t('auth.createAccountSubtitle') }}
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
          :label="$t('auth.fullName')"
          type="text"
          placeholder="Ada Lovelace"
          required
        />

        <AppInput
          id="email"
          v-model="email"
          :label="$t('auth.emailWorkOrPersonal')"
          type="email"
          placeholder="engineer@company.com"
          required
        />

        <AppInput
          id="password"
          v-model="password"
          :label="$t('auth.passwordHint')"
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
          {{ $t('auth.createAccountButton') }}
        </AppButton>
      </form>

      <div class="pt-4 border-t border-slate-200 dark:border-slate-800 text-center text-xs text-slate-500">
        {{ $t('auth.alreadyHaveAccount') }}
        <NuxtLink to="/login" class="text-indigo-600 dark:text-indigo-400 hover:underline font-semibold ml-1">
          {{ $t('auth.signInHere') }}
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
