<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useAuthService } from '~/features/auth/services/auth.service'

const route = useRoute()
const router = useRouter()
const { user, isAuthenticated, logout } = useAuthService()
const isMobileMenuOpen = ref(false)

const handleLogout = async () => {
  await logout()
  isMobileMenuOpen.value = false
  await router.push('/login')
}

router.afterEach(() => {
  isMobileMenuOpen.value = false
})
</script>
<template>
  <div class="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 font-sans selection:bg-indigo-500 selection:text-white">
    
    <header class="border-b border-slate-200 dark:border-slate-800 bg-white/90 dark:bg-slate-900/90 backdrop-blur sticky top-0 z-40">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-14 flex items-center justify-between">
        <div class="flex items-center gap-6">
          <NuxtLink to="/dashboard" class="flex items-center gap-2.5 group">
            <div class="w-7 h-7 rounded bg-indigo-600 flex items-center justify-center text-white font-bold text-sm tracking-wider shadow-sm group-hover:bg-indigo-500 transition-colors">
              面
            </div>
            <span class="font-semibold text-slate-900 dark:text-white tracking-tight">Mensetsu</span>
          </NuxtLink>

          <nav class="hidden md:flex items-center gap-1">
            <NuxtLink
              to="/dashboard"
              class="px-3 py-1.5 text-xs font-medium rounded-md transition"
              :class="route.path === '/dashboard' ? 'bg-slate-100 dark:bg-slate-800 text-indigo-600 dark:text-indigo-400 font-semibold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            >
              Dashboard
            </NuxtLink>
            <NuxtLink
              to="/interviews"
              class="px-3 py-1.5 text-xs font-medium rounded-md transition"
              :class="route.path.startsWith('/interviews') ? 'bg-slate-100 dark:bg-slate-800 text-indigo-600 dark:text-indigo-400 font-semibold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            >
              Interviews
            </NuxtLink>
            <NuxtLink
              v-if="user?.role === 'admin'"
              to="/admin/pipelines"
              class="px-3 py-1.5 text-xs font-medium rounded-md transition"
              :class="route.path.startsWith('/admin') ? 'bg-slate-100 dark:bg-slate-800 text-indigo-600 dark:text-indigo-400 font-semibold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            >
              Admin Pipelines
            </NuxtLink>
          </nav>
        </div>

        <div class="hidden sm:flex items-center gap-3">
          <template v-if="isAuthenticated && user">
            <NuxtLink
              to="/interviews/new"
              class="inline-flex items-center justify-center px-3.5 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
            >
              + New Run
            </NuxtLink>

            <div class="flex items-center gap-2 pl-2 border-l border-slate-200 dark:border-slate-800">
              <div class="w-7 h-7 rounded-full bg-slate-200 dark:bg-slate-700 flex items-center justify-center text-xs font-semibold text-slate-700 dark:text-slate-200 uppercase">
                {{ user.name?.charAt(0) || user.email.charAt(0) }}
              </div>
              <span class="text-xs font-medium text-slate-700 dark:text-slate-200 hidden sm:inline">
                {{ user.name || user.email }}
              </span>
              <button
                type="button"
                class="ml-1 px-2 py-1 text-xs text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 font-medium transition"
                @click="handleLogout"
              >
                Sign out
              </button>
            </div>
          </template>
          <template v-else>
            <NuxtLink
              to="/login"
              class="px-3 py-1.5 text-xs font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white transition"
            >
              Sign in
            </NuxtLink>
            <NuxtLink
              to="/register"
              class="inline-flex items-center justify-center px-3.5 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
            >
              Get Started
            </NuxtLink>
          </template>
        </div>

        <!-- Mobile Hamburger Button -->
        <div class="flex items-center sm:hidden gap-2">
          <button
            type="button"
            aria-label="Toggle Navigation Menu"
            class="p-2 rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 focus:outline-none"
            @click="isMobileMenuOpen = !isMobileMenuOpen"
          >
            <svg v-if="!isMobileMenuOpen" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
            <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Mobile Navigation Drawer -->
      <div v-if="isMobileMenuOpen" class="sm:hidden border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 px-4 pt-3 pb-4 space-y-2 shadow-lg">
        <template v-if="isAuthenticated && user">
          <div class="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
            <div class="w-8 h-8 rounded-full bg-indigo-100 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-400 flex items-center justify-center text-xs font-bold uppercase">
              {{ user.name?.charAt(0) || user.email.charAt(0) }}
            </div>
            <div class="min-w-0 flex-1">
              <div class="text-xs font-semibold text-slate-900 dark:text-white truncate">{{ user.name || user.email }}</div>
              <div class="text-[11px] text-slate-500 font-mono">{{ user.role }}</div>
            </div>
          </div>
          <NuxtLink to="/dashboard" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
            Dashboard
          </NuxtLink>
          <NuxtLink to="/interviews" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
            Interviews
          </NuxtLink>
          <NuxtLink v-if="user.role === 'admin'" to="/admin/pipelines" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
            Pipelines Orchestration
          </NuxtLink>
          <NuxtLink to="/settings" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
            Settings & 2FA
          </NuxtLink>
          <NuxtLink to="/interviews/new" class="block text-center mt-2 px-3 py-2 rounded-md text-xs font-semibold bg-indigo-600 text-white">
            + New Preparation Run
          </NuxtLink>
          <button
            type="button"
            class="w-full text-left px-3 py-2 rounded-md text-xs font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30"
            @click="handleLogout"
          >
            Sign out
          </button>
        </template>
        <template v-else>
          <NuxtLink to="/login" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
            Sign in
          </NuxtLink>
          <NuxtLink to="/register" class="block text-center mt-2 px-3 py-2 rounded-md text-xs font-semibold bg-indigo-600 text-white">
            Get Started
          </NuxtLink>
        </template>
      </div>
    </header>
    
    <main class="flex-1">
      <slot />
    </main>

    
    <footer class="border-t border-slate-200 dark:border-slate-800 py-6 text-center text-xs text-slate-500 dark:text-slate-500">
      Mensetsu — Evidence-Backed Technical Interview Intelligence
    </footer>
  </div>
</template>
