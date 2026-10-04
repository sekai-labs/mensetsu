<script setup lang="ts">
import { ref } from 'vue'
import { useAuthService } from '~/features/auth/services/auth.service'
import type { TotpSetupData } from '~/features/auth/models/auth.model'

const authService = useAuthService()
const user = authService.user

const isSettingUp = ref(false)
const setupData = ref<TotpSetupData | null>(null)
const verificationCode = ref('')
const disableCode = ref('')
const isDisabling = ref(false)
const errorMsg = ref('')
const successMsg = ref('')
const loading = ref(false)

const handleStartSetup = async () => {
  errorMsg.value = ''
  successMsg.value = ''
  loading.value = true
  try {
    setupData.value = await authService.setupTotp()
    isSettingUp.value = true
    verificationCode.value = ''
  } catch (err: unknown) {
    const errorObj = err as { message?: string }
    errorMsg.value = errorObj.message || 'Failed to initialize 2FA setup'
  } finally {
    loading.value = false
  }
}

const handleVerifySetup = async () => {
  if (verificationCode.value.length !== 6) {
    errorMsg.value = 'Please enter a 6-digit code'
    return
  }
  errorMsg.value = ''
  loading.value = true
  try {
    await authService.verifyTotp(verificationCode.value)
    isSettingUp.value = false
    setupData.value = null
    successMsg.value = 'Two-factor authentication successfully enabled!'
  } catch (err: unknown) {
    const errorObj = err as { message?: string }
    errorMsg.value = errorObj.message || 'Invalid verification code'
  } finally {
    loading.value = false
  }
}

const handleDisable2fa = async () => {
  if (disableCode.value.length !== 6) {
    errorMsg.value = 'Please enter a 6-digit code from your authenticator'
    return
  }
  errorMsg.value = ''
  loading.value = true
  try {
    await authService.disableTotp(disableCode.value)
    isDisabling.value = false
    disableCode.value = ''
    successMsg.value = 'Two-factor authentication has been disabled.'
  } catch (err: unknown) {
    const errorObj = err as { message?: string }
    errorMsg.value = errorObj.message || 'Failed to disable 2FA'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-5">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-4 border-b border-slate-100 dark:border-slate-800">
      <div>
        <h3 class="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
          Two-Factor Authentication (TOTP)
          <span
            v-if="user?.totp_enabled"
            class="text-[10px] px-2 py-0.5 rounded-full font-mono font-semibold uppercase bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800"
          >
            Active
          </span>
          <span
            v-else
            class="text-[10px] px-2 py-0.5 rounded-full font-mono font-semibold uppercase bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400 border border-slate-200 dark:border-slate-700"
          >
            Disabled
          </span>
        </h3>
        <p class="text-xs text-slate-500 mt-1">
          Protect your account using standard TOTP apps (Google Authenticator, 1Password, Bitwarden, Authy).
        </p>
      </div>

      <div>
        <button
          v-if="!user?.totp_enabled && !isSettingUp"
          @click="handleStartSetup"
          :disabled="loading"
          class="px-3.5 py-1.5 text-xs font-semibold rounded bg-indigo-600 hover:bg-indigo-500 text-white transition disabled:opacity-50"
        >
          Enable 2FA
        </button>

        <button
          v-else-if="user?.totp_enabled && !isDisabling"
          @click="isDisabling = true"
          :disabled="loading"
          class="px-3.5 py-1.5 text-xs font-semibold rounded border border-rose-200 dark:border-rose-900 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 transition disabled:opacity-50"
        >
          Disable 2FA
        </button>
      </div>
    </div>

    <div v-if="successMsg" class="p-3 text-xs bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800 rounded text-emerald-800 dark:text-emerald-300">
      {{ successMsg }}
    </div>

    <div v-if="errorMsg" class="p-3 text-xs bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 rounded text-rose-800 dark:text-rose-300">
      {{ errorMsg }}
    </div>

    <!-- Setup Modal / Inset -->
    <div v-if="isSettingUp && setupData" class="p-4 rounded-lg bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700 space-y-4">
      <h4 class="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Scan QR Code with Authenticator App
      </h4>
      <div class="flex flex-col sm:flex-row items-center gap-5">
        <div class="p-2 bg-white rounded border border-slate-200 shadow-sm shrink-0">
          <img :src="setupData.qr_code_data_url" alt="2FA QR Code" class="w-36 h-36" />
        </div>
        <div class="space-y-2 text-xs">
          <p class="text-slate-600 dark:text-slate-400">
            If you cannot scan the QR code, manually enter this secret into your authenticator app:
          </p>
          <div class="p-2 rounded bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 font-mono text-[11px] text-indigo-600 dark:text-indigo-400 break-all select-all">
            {{ setupData.secret }}
          </div>
        </div>
      </div>

      <div class="pt-2 border-t border-slate-200 dark:border-slate-700 flex flex-col sm:flex-row items-center gap-3">
        <input
          v-model="verificationCode"
          type="text"
          maxlength="6"
          placeholder="000000"
          class="w-36 px-3 py-1.5 text-center font-mono tracking-widest text-sm rounded border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500"
          @keydown.enter.prevent="handleVerifySetup"
        />
        <button
          @click="handleVerifySetup"
          :disabled="loading || verificationCode.length !== 6"
          class="px-4 py-1.5 text-xs font-semibold rounded bg-emerald-600 hover:bg-emerald-500 text-white transition disabled:opacity-50"
        >
          Confirm & Activate
        </button>
        <button
          @click="isSettingUp = false"
          class="px-3 py-1.5 text-xs font-medium rounded border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition"
        >
          Cancel
        </button>
      </div>
    </div>

    <!-- Disable confirmation inset -->
    <div v-if="isDisabling" class="p-4 rounded-lg bg-rose-50/50 dark:bg-rose-950/20 border border-rose-200 dark:border-rose-900 space-y-3">
      <h4 class="text-xs font-bold uppercase tracking-wider text-rose-700 dark:text-rose-400">
        Confirm Disabling Two-Factor Authentication
      </h4>
      <p class="text-xs text-slate-600 dark:text-slate-400">
        Enter the current 6-digit TOTP code from your authenticator app to confirm:
      </p>
      <div class="flex items-center gap-3">
        <input
          v-model="disableCode"
          type="text"
          maxlength="6"
          placeholder="000000"
          class="w-36 px-3 py-1.5 text-center font-mono tracking-widest text-sm rounded border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-rose-500"
          @keydown.enter.prevent="handleDisable2fa"
        />
        <button
          @click="handleDisable2fa"
          :disabled="loading || disableCode.length !== 6"
          class="px-4 py-1.5 text-xs font-semibold rounded bg-rose-600 hover:bg-rose-500 text-white transition disabled:opacity-50"
        >
          Confirm Disable
        </button>
        <button
          @click="isDisabling = false"
          class="px-3 py-1.5 text-xs font-medium rounded border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>
