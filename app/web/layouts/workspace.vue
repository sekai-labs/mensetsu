<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { useInterviewEventsService } from '~/features/interviews/services/interview-events.service'
import { useAuthService } from '~/features/auth/services/auth.service'
import StatusBadge from '~/components/ui/StatusBadge.vue'
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
const workspaceId = computed(() => String(route.params.id || ''))

const { getWorkspace } = useInterviewService()
const { data: workspace, refresh: refreshWorkspace } = await useAsyncData(
  () => `workspace-${workspaceId.value}`,
  () => (workspaceId.value ? getWorkspace(workspaceId.value) : Promise.resolve(null)),
  { watch: [workspaceId] }
)

const { currentPhase, progressPercent, status: sseStatus } = useInterviewEventsService(workspaceId.value, () => {
  refreshWorkspace()
})

const effectiveStatus = computed(() => sseStatus.value || workspace.value?.status || 'pending')
const effectivePercent = computed(() => progressPercent.value || workspace.value?.progress_percent || 0)
const effectivePhase = computed(() => currentPhase.value || workspace.value?.current_phase || 'preparing_input')

const navItems = [
  { name: 'Overview', path: 'overview', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
  { name: 'Match Analysis', path: 'match', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
  { name: 'Company', path: 'company', icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4' },
  { name: 'Role & JD', path: 'role', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2' },
  { name: 'Skill Map', path: 'skills', icon: 'M13 10V3L4 14h7v7l9-11h-7z' },
  { name: 'Knowledge Graph', path: 'knowledge', icon: 'M4 7v10c0 2 1 3 3 3h10c2 0 3-1 3-3V7c0-2-1-3-3-3H7C5 4 4 5 4 7zm0 5h16M9 4v16' },
  { name: 'Learning Plan', path: 'learning', icon: 'M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253' },
  { name: 'Question Book', path: 'questions', icon: 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
  { name: 'Practice', path: 'practice', icon: 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z' },
  { name: 'Sources & Evidence', path: 'sources', icon: 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1' },
  { name: 'Settings', path: 'settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z' },
]

const isActive = (itemPath: string) => {
  return route.path === `/interviews/${workspaceId.value}/${itemPath}`
}
</script>

<template>
  <div class="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 font-sans selection:bg-indigo-500 selection:text-white">
    
    <header class="border-b border-slate-200 dark:border-slate-800 bg-white/90 dark:bg-slate-900/90 backdrop-blur sticky top-0 z-40">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-14 flex items-center justify-between">
        <div class="flex items-center gap-3 min-w-0">
          <NuxtLink to="/dashboard" class="flex items-center gap-2 group shrink-0">
            <div class="w-6 h-6 rounded bg-indigo-600 flex items-center justify-center text-white font-bold text-xs tracking-wider shadow-sm group-hover:bg-indigo-500 transition-colors">
              面
            </div>
            <span class="font-semibold text-slate-900 dark:text-white tracking-tight text-sm hidden sm:inline">Mensetsu</span>
          </NuxtLink>

          <span class="text-slate-300 dark:text-slate-700 hidden sm:inline">/</span>

          <NuxtLink to="/interviews" class="text-xs font-medium text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition hidden sm:inline">
            Interviews
          </NuxtLink>

          <span class="text-slate-300 dark:text-slate-700 hidden sm:inline">/</span>

          <div class="flex items-center gap-1.5 min-w-0">
            <span class="text-xs font-semibold text-slate-900 dark:text-white truncate max-w-[120px] sm:max-w-xs">
              {{ workspace?.company_name || 'Workspace' }}
            </span>
            <span class="text-xs text-slate-400 truncate hidden md:inline">({{ workspace?.role_title }})</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <StatusBadge :status="effectiveStatus" size="sm" />
          <span class="text-xs font-mono text-slate-500 hidden sm:inline">{{ effectivePercent }}%</span>
          <NuxtLink
            to="/interviews/new"
            class="px-3 py-1.5 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
          >
            + New
          </NuxtLink>
          <div v-if="isAuthenticated && user" class="hidden sm:flex items-center gap-2.5 pl-3 border-l border-slate-200 dark:border-slate-800">
            <div class="w-7 h-7 rounded-full bg-slate-200 dark:bg-slate-700 flex items-center justify-center text-xs font-semibold text-slate-700 dark:text-slate-200 uppercase">
              {{ user.name?.charAt(0) || user.email.charAt(0) }}
            </div>
            <NuxtLink
              to="/settings"
              class="text-xs text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 font-medium transition"
            >
              Settings
            </NuxtLink>
            <button
              type="button"
              class="ml-1 px-2 py-1 text-xs text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 font-medium transition"
              @click="handleLogout"
            >
              Sign out
            </button>
          </div>

          <!-- Mobile Menu Toggle Button -->
          <button
            type="button"
            aria-label="Toggle Navigation Menu"
            class="p-1.5 sm:hidden rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
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

      <!-- Mobile Drawer for Profile & Common Links -->
      <div v-if="isMobileMenuOpen" class="sm:hidden border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 px-4 pt-3 pb-4 space-y-2 shadow-lg">
        <div class="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
          <div class="w-8 h-8 rounded-full bg-indigo-100 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-400 flex items-center justify-center text-xs font-bold uppercase">
            {{ user?.name?.charAt(0) || user?.email.charAt(0) || 'U' }}
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-xs font-semibold text-slate-900 dark:text-white truncate">{{ user?.name || user?.email }}</div>
            <div class="text-[11px] text-slate-500 font-mono">{{ user?.role }}</div>
          </div>
        </div>
        <NuxtLink to="/dashboard" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
          Dashboard
        </NuxtLink>
        <NuxtLink to="/interviews" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
          All Interviews
        </NuxtLink>
        <NuxtLink to="/settings" class="block px-3 py-2 rounded-md text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800">
          Settings & 2FA
        </NuxtLink>
        <button
          type="button"
          class="w-full text-left px-3 py-2 rounded-md text-xs font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30"
          @click="handleLogout"
        >
          Sign out
        </button>
      </div>

      <!-- Horizontal Mobile Tabs for Workspace Navigation -->
      <div class="md:hidden border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 px-2 py-1 overflow-x-auto flex items-center gap-1 scrollbar-none">
        <NuxtLink
          v-for="item in navItems"
          :key="item.path"
          :to="`/interviews/${workspaceId}/${item.path}`"
          class="px-2.5 py-1.5 rounded text-xs whitespace-nowrap font-medium transition shrink-0"
          :class="isActive(item.path)
            ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-semibold'
            : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
        >
          {{ item.name }}
        </NuxtLink>
      </div>
    </header>
    
    <div
      v-if="effectiveStatus !== 'ready' && effectiveStatus !== 'failed'"
      class="bg-indigo-50 dark:bg-indigo-950/40 border-b border-indigo-100 dark:border-indigo-900/50 px-4 py-2 text-xs flex items-center justify-between"
    >
      <div class="flex items-center gap-2 max-w-7xl mx-auto w-full">
        <span class="w-2 h-2 rounded-full bg-indigo-600 animate-ping"></span>
        <span class="text-indigo-900 dark:text-indigo-200">
          Durable background research running: <strong class="capitalize">{{ effectivePhase.replace(/_/g, ' ') }}</strong> ({{ effectivePercent }}%)
        </span>
      </div>
    </div>

    
    <div class="flex-1 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-6 flex flex-col md:flex-row gap-6">
      
      <aside class="hidden md:block w-56 shrink-0">
        <div class="sticky top-20 bg-white dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800 p-2 shadow-sm">
          <div class="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-slate-400">
            Workspace Hub
          </div>
          <nav class="space-y-0.5">
            <NuxtLink
              v-for="item in navItems"
              :key="item.path"
              :to="`/interviews/${workspaceId}/${item.path}`"
              class="flex items-center gap-2.5 px-3 py-2 rounded-md text-xs font-medium transition"
              :class="isActive(item.path)
                ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-semibold'
                : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-slate-900 dark:hover:text-white'"
            >
              <svg class="w-4 h-4 shrink-0 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="item.icon" />
              </svg>
              <span>{{ item.name }}</span>
            </NuxtLink>
          </nav>
        </div>
      </aside>

      
      <section class="flex-1 min-w-0">
        <slot />
      </section>
    </div>

    
    <footer class="border-t border-slate-200 dark:border-slate-800 py-6 text-center text-xs text-slate-500">
      Mensetsu — Evidence-Backed Technical Interview Intelligence
    </footer>
  </div>
</template>
