<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { onClickOutside, useEventListener } from '@vueuse/core'
import {
  ChevronDown,
  LayoutDashboard,
  Workflow,
  LogOut,
  Settings,
  Shield,
} from 'lucide-vue-next'
import { useAuthService } from '~/features/auth/services/auth.service'

const router = useRouter()
const { user, logout, isAuthenticated } = useAuthService()

const isOpen = ref(false)
const containerRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)

const toggleOpen = () => {
  isOpen.value = !isOpen.value
}

const close = () => {
  if (isOpen.value) {
    isOpen.value = false
    triggerRef.value?.focus()
  }
}

onClickOutside(containerRef, () => {
  if (isOpen.value) {
    isOpen.value = false
  }
})

useEventListener('keydown', (e: KeyboardEvent) => {
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
})

const userInitials = computed(() => {
  if (!user.value) return '?'
  if (user.value.name) {
    const parts = user.value.name.trim().split(/\s+/)
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase()
    }
    return parts[0].slice(0, 2).toUpperCase()
  }
  if (user.value.email) {
    return user.value.email.slice(0, 2).toUpperCase()
  }
  return 'U'
})

const handleLogout = async () => {
  isOpen.value = false
  await logout()
  await router.push('/login')
}
</script>

<template>
  <div v-if="isAuthenticated && user" ref="containerRef" class="relative inline-block text-left">

    <button
      ref="triggerRef"
      type="button"
      class="flex items-center gap-2 p-1 pl-1.5 pr-2 rounded-lg border border-transparent hover:border-slate-200 dark:hover:border-slate-800 hover:bg-slate-100 dark:hover:bg-slate-800/60 transition-all focus:outline-none focus:ring-2 focus:ring-indigo-500"
      :class="isOpen ? 'bg-slate-100 dark:bg-slate-800 border-slate-200 dark:border-slate-700' : ''"
      :aria-expanded="isOpen"
      aria-haspopup="menu"
      aria-label="User Account Menu"
      @click="toggleOpen"
    >

      <div class="w-6 h-6 rounded bg-indigo-600 text-white font-mono text-[11px] font-semibold flex items-center justify-center shrink-0">
        {{ userInitials }}
      </div>


      <span class="text-xs font-medium text-slate-700 dark:text-slate-300 hidden md:inline truncate max-w-[120px]">
        {{ user.name || user.email }}
      </span>

      <ChevronDown
        class="w-3.5 h-3.5 text-slate-400 shrink-0 transition-transform duration-150"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>


    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-56 rounded-lg bg-white dark:bg-slate-900 shadow-xl border border-slate-200 dark:border-slate-800 py-1.5 z-50 text-left animate-in fade-in zoom-in-95 duration-100"
      role="menu"
      aria-orientation="vertical"
    >

      <div class="px-3.5 py-2 border-b border-slate-100 dark:border-slate-800 mb-1">
        <div class="text-xs font-semibold text-slate-900 dark:text-white truncate">
          {{ user.name || 'Engineer' }}
        </div>
        <div class="text-[11px] font-mono text-slate-500 dark:text-slate-400 truncate">
          {{ user.email }}
        </div>
        <div v-if="user.role" class="mt-1">
          <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300">
            {{ user.role }}
          </span>
        </div>
      </div>


      <NuxtLink
        to="/dashboard"
        class="flex items-center gap-2.5 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/70 hover:text-slate-900 dark:hover:text-white transition-colors"
        role="menuitem"
        @click="isOpen = false"
      >
        <LayoutDashboard class="w-4 h-4 text-slate-400" />
        <span>{{ $t('navigation.items.overview') }}</span>
      </NuxtLink>
      <NuxtLink
        to="/workflows"
        class="flex items-center gap-2.5 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/70 hover:text-slate-900 dark:hover:text-white transition-colors"
        role="menuitem"
        @click="isOpen = false"
      >
        <Workflow class="w-4 h-4 text-slate-400" />
        <span>{{ $t('navigation.workflows') }}</span>
      </NuxtLink>

      <NuxtLink
        to="/settings"
        class="flex items-center gap-2.5 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/70 hover:text-slate-900 dark:hover:text-white transition-colors"
        role="menuitem"
        @click="isOpen = false"
      >
        <Settings class="w-4 h-4 text-slate-400" />
        <span>{{ $t('navigation.accountAnd2fa') }}</span>
      </NuxtLink>

      <NuxtLink
        v-if="user.role === 'admin'"
        to="/admin/pipelines"
        class="flex items-center gap-2.5 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/70 hover:text-slate-900 dark:hover:text-white transition-colors"
        role="menuitem"
        @click="isOpen = false"
      >
        <Shield class="w-4 h-4 text-slate-400" />
        <span>{{ $t('navigation.pipelineAdmin') }}</span>
      </NuxtLink>


      <div class="my-1 border-t border-slate-100 dark:border-slate-800" />


      <button
        type="button"
        class="w-full flex items-center gap-2.5 px-3 py-1.5 text-xs text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors"
        role="menuitem"
        @click="handleLogout"
      >
        <LogOut class="w-4 h-4 text-rose-500" />
        <span>{{ $t('navigation.signOut') }}</span>
      </button>
    </div>
  </div>


  <div v-else class="flex items-center gap-2">
    <NuxtLink
      to="/login"
      class="text-xs font-medium text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white transition-colors px-2.5 py-1"
    >
      {{ $t('navigation.signIn') }}
    </NuxtLink>
    <NuxtLink
      to="/register"
      class="text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-500 transition-colors px-2.5 py-1 rounded-md shadow-sm"
    >
      {{ $t('navigation.register') }}
    </NuxtLink>
  </div>
</template>
