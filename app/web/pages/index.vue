<script setup lang="ts">
import { computed } from 'vue'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import { useAuthService } from '~/features/auth/services/auth.service'
import StatusBadge from '~/components/ui/StatusBadge.vue'

const { listWorkspaces } = useInterviewService()
const { isAuthenticated } = useAuthService()

const { data: workspaces, pending, error } = await useAsyncData(
  'landing-workspaces',
  async () => {
    if (!isAuthenticated.value) return []
    try {
      return await listWorkspaces()
    } catch {
      return []
    }
  },
  { watch: [isAuthenticated] }
)

const activeCount = computed(() => workspaces.value?.filter(w => w.status === 'processing' || w.status === 'pending').length ?? 0)
const readyCount = computed(() => workspaces.value?.filter(w => w.status === 'ready').length ?? 0)
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-12">
    
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 border-b border-slate-200 dark:border-slate-800 pb-8">
      <div class="max-w-3xl">
        <div class="inline-flex items-center gap-2 px-2.5 py-1 rounded-full text-xs font-medium bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-400 border border-indigo-200 dark:border-indigo-800 mb-3">
          <span>面接</span>
          <span>•</span>
          <span>Technical Interview Workspace</span>
        </div>
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white sm:text-4xl">
          Evidence-Backed Interview Preparation
        </h1>
        <p class="text-sm text-slate-600 dark:text-slate-400 mt-2 leading-relaxed">
          Ground your preparation in verified company research, job requirements, and structured mental models. No generic chat, no hallucinations.
        </p>

        <div class="flex items-center gap-3 mt-6">
          <NuxtLink
            v-if="isAuthenticated"
            to="/interviews/new"
            class="inline-flex items-center px-4 py-2 text-sm font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
          >
            + Create Preparation Workspace
          </NuxtLink>
          <NuxtLink
            v-if="isAuthenticated"
            to="/dashboard"
            class="inline-flex items-center px-4 py-2 text-sm font-semibold rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition shadow-sm"
          >
            Go to Dashboard →
          </NuxtLink>
          <NuxtLink
            v-if="!isAuthenticated"
            to="/register"
            class="inline-flex items-center px-4 py-2 text-sm font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
          >
            Get Started Free
          </NuxtLink>
          <NuxtLink
            v-if="!isAuthenticated"
            to="/login"
            class="inline-flex items-center px-4 py-2 text-sm font-semibold rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition shadow-sm"
          >
            Sign In →
          </NuxtLink>
        </div>
      </div>

      <div class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm min-w-[260px] space-y-3">
        <div class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Quick Workspace Status</div>
        <div class="flex items-baseline justify-between">
          <span class="text-xs text-slate-600 dark:text-slate-400">Ready Workspaces:</span>
          <span class="font-mono font-bold text-emerald-600 dark:text-emerald-400">{{ readyCount }}</span>
        </div>
        <div class="flex items-baseline justify-between">
          <span class="text-xs text-slate-600 dark:text-slate-400">Pipelines Running:</span>
          <span class="font-mono font-bold text-indigo-600 dark:text-indigo-400">{{ activeCount }}</span>
        </div>
        <div class="flex items-baseline justify-between pt-2 border-t border-slate-100 dark:border-slate-800">
          <span class="text-xs text-slate-600 dark:text-slate-400">Total Preps:</span>
          <span class="font-mono font-bold text-slate-900 dark:text-white">{{ workspaces?.length ?? 0 }}</span>
        </div>
      </div>
    </div>

    
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-bold tracking-tight text-slate-900 dark:text-white">Recent Workspaces</h2>
          <p class="text-xs text-slate-500">Pick up your structured study path where you left off.</p>
        </div>
        <NuxtLink to="/interviews" class="text-xs font-semibold text-indigo-600 hover:text-indigo-500 transition">
          View all workspaces →
        </NuxtLink>
      </div>

      <div v-if="pending" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div v-for="i in 3" :key="i" class="h-44 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 animate-pulse p-6"></div>
      </div>

      <div v-else-if="error" class="p-6 rounded-lg bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-400 text-sm">
        Failed to load workspaces. Please ensure the Mensetsu Core API is running.
      </div>

      <div v-else-if="!workspaces || workspaces.length === 0" class="text-center py-16 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg">
        <h3 class="text-base font-medium text-slate-900 dark:text-white">No interview workspaces yet</h3>
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-1 mb-5">
          Upload your resume and enter a target company to begin an asynchronous research run.
        </p>
        <NuxtLink
          to="/interviews/new"
          class="inline-flex items-center px-4 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition"
        >
          Create First Workspace
        </NuxtLink>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <NuxtLink
          v-for="ws in workspaces"
          :key="ws.id"
          :to="`/interviews/${ws.id}/overview`"
          class="group block p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 hover:border-slate-300 dark:hover:border-slate-700 hover:shadow-sm transition-all"
        >
          <div class="flex items-start justify-between">
            <StatusBadge :status="ws.status" size="sm" />
            <span class="text-xs text-slate-400 font-mono">{{ ws.progress_percent }}%</span>
          </div>

          <h3 class="text-lg font-semibold text-slate-900 dark:text-white mt-4 group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
            {{ ws.company_name }}
          </h3>
          <p class="text-sm text-slate-600 dark:text-slate-400 mt-1 font-medium">{{ ws.role_title }}</p>

          <div class="mt-5 pt-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs text-slate-400">
            <span class="capitalize">{{ ws.current_phase.replace(/_/g, ' ') }}</span>
            <span>Open Workspace →</span>
          </div>
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
