<script setup lang="ts">
import { useRoute } from 'vue-router'
import { WORKSPACE_NAV_SECTIONS } from './types'
import type { WorkspaceResponse } from '~/features/interviews/models/interview.model'
import StatusBadge from '~/components/ui/StatusBadge.vue'

interface Props {
  workspaceId: string
  workspace?: WorkspaceResponse | null
  status?: string
}

const props = withDefaults(defineProps<Props>(), {
  workspace: null,
  status: 'ready',
})

const route = useRoute()

const { t, te } = useI18n()

const isRouteActive = (subPath: string) => {
  const current = route.path
  const target = `/interviews/${props.workspaceId}/${subPath}`
  return current === target || current.startsWith(`${target}/`)
}

const itemKeyOrFallback = (key?: string, fallback?: string) => {
  if (!key) return fallback || ''
  return te(key) ? t(key) : (fallback || key)
}
</script>

<template>
  <aside
    class="w-60 shrink-0"
    aria-label="Workspace Navigation"
  >
    <div class="sticky top-18 space-y-5">

      <div
        v-if="workspace"
        class="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900/60 shadow-xs space-y-2"
      >
        <div class="flex items-center justify-between gap-1">
          <span class="text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 font-semibold">
            {{ $t('navigation.targetRole') }}
          </span>
          <StatusBadge :status="status" size="xs" :show-dot="true" />
        </div>

        <div class="min-w-0">
          <div class="text-xs font-semibold text-slate-900 dark:text-white truncate" :title="workspace.company_name">
            {{ workspace.company_name }}
          </div>
          <div class="text-[11px] text-slate-500 dark:text-slate-400 truncate" :title="workspace.role_title">
            {{ workspace.role_title }}
          </div>
        </div>

        <div v-if="workspace.seniority" class="pt-1 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-[10px] font-mono text-slate-400">
          <span>Level</span>
          <span class="uppercase font-semibold text-slate-600 dark:text-slate-300">
            {{ workspace.seniority }}
          </span>
        </div>
      </div>


      <nav class="space-y-4">
        <div
          v-for="section in WORKSPACE_NAV_SECTIONS"
          :key="section.title"
          class="space-y-1"
        >

          <h2 class="px-2.5 text-[10px] font-mono font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
            {{ itemKeyOrFallback(section.titleKey, section.title) }}
          </h2>


          <div class="space-y-0.5">
            <NuxtLink
              v-for="item in section.items"
              :key="item.id"
              :to="`/interviews/${workspaceId}/${item.path}`"
              class="group flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-xs font-medium transition-all"
              :class="[
                isRouteActive(item.path)
                  ? 'bg-slate-100 dark:bg-slate-800/60 font-semibold text-slate-900 dark:text-white border-l-2 border-indigo-600 pl-2'
                  : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100/70 dark:hover:bg-slate-800/40 hover:text-slate-900 dark:hover:text-white',
              ]"
              :aria-current="isRouteActive(item.path) ? 'page' : undefined"
            >
              <component
                :is="item.icon"
                class="w-4 h-4 shrink-0 transition-colors"
                :class="[
                  isRouteActive(item.path)
                    ? 'text-indigo-600 dark:text-indigo-400'
                    : 'text-slate-400 dark:text-slate-500 group-hover:text-slate-600 dark:group-hover:text-slate-300',
                ]"
              />
              <span class="truncate">{{ itemKeyOrFallback(item.nameKey, item.name) }}</span>

              <span
                v-if="item.badge"
                class="ml-auto text-[10px] font-mono px-1.5 py-0.2 rounded bg-slate-200 dark:bg-slate-800 text-slate-600 dark:text-slate-300"
              >
                {{ item.badge }}
              </span>
            </NuxtLink>
          </div>
        </div>


        <div class="pt-2 border-t border-slate-200 dark:border-slate-800">
          <NuxtLink
            to="/workflows"
            class="group flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-xs font-medium text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-white transition-all"
          >
            <span class="font-mono text-slate-500 dark:text-slate-400 font-bold">⚡</span>
            <span class="truncate">{{ $t('navigation.workflowRuns') }}</span>
          </NuxtLink>
        </div>
      </nav>
    </div>
  </aside>
</template>
