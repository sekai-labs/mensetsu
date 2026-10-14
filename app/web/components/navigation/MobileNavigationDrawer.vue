<script setup lang="ts">
import { watch } from 'vue'
import { useRoute } from 'vue-router'
import { useEventListener } from '@vueuse/core'
import { X } from 'lucide-vue-next'
import { WORKSPACE_NAV_SECTIONS } from './types'
import type { WorkspaceResponse } from '~/features/interviews/models/interview.model'
import StatusBadge from '~/components/ui/StatusBadge.vue'

interface Props {
  isOpen: boolean
  workspaceId: string
  workspace?: WorkspaceResponse | null
  status?: string
}

const props = withDefaults(defineProps<Props>(), {
  isOpen: false,
  workspace: null,
  status: 'ready',
})

const emit = defineEmits<{
  (e: 'close'): void
}>()

const route = useRoute()


watch(
  () => route.path,
  () => {
    if (props.isOpen) {
      emit('close')
    }
  }
)


useEventListener('keydown', (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.isOpen) {
    emit('close')
  }
})

const isRouteActive = (subPath: string) => {
  const current = route.path
  const target = `/interviews/${props.workspaceId}/${subPath}`
  return current === target || current.startsWith(`${target}/`)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 lg:hidden flex"
      role="dialog"
      aria-modal="true"
      aria-label="Workspace Navigation Drawer"
    >

      <div
        class="fixed inset-0 bg-slate-900/60 transition-opacity animate-in fade-in duration-200"
        aria-hidden="true"
        @click="emit('close')"
      />


      <div
        class="relative flex-1 flex flex-col max-w-xs w-full bg-white dark:bg-slate-900 border-r border-slate-200 dark:border-slate-800 shadow-2xl transition-transform animate-in slide-in-from-left duration-200"
      >

        <div class="h-14 px-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <div class="w-6 h-6 rounded bg-indigo-600 flex items-center justify-center text-white font-bold text-xs tracking-wider">
              面
            </div>
            <span class="font-semibold text-slate-900 dark:text-white text-sm">
              Mensetsu
            </span>
          </div>

          <button
            type="button"
            class="p-1.5 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            aria-label="Close navigation drawer"
            @click="emit('close')"
          >
            <X class="w-5 h-5" />
          </button>
        </div>


        <div v-if="workspace" class="p-4 border-b border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/40">
          <div class="flex items-center justify-between gap-1 mb-1">
            <span class="text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 font-semibold">
              Current Target
            </span>
            <StatusBadge :status="status" size="xs" :show-dot="true" />
          </div>
          <div class="text-xs font-semibold text-slate-900 dark:text-white truncate">
            {{ workspace.company_name }}
          </div>
          <div class="text-[11px] text-slate-500 dark:text-slate-400 truncate">
            {{ workspace.role_title }}
          </div>
        </div>


        <div class="flex-1 overflow-y-auto px-3 py-4 space-y-4">
          <div
            v-for="section in WORKSPACE_NAV_SECTIONS"
            :key="section.title"
            class="space-y-1"
          >
            <div class="px-2 text-[10px] font-mono font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
              {{ section.title }}
            </div>

            <nav class="space-y-0.5">
              <NuxtLink
                v-for="item in section.items"
                :key="item.id"
                :to="`/interviews/${workspaceId}/${item.path}`"
                class="group flex items-center gap-2.5 px-2.5 py-2 rounded-md text-xs font-medium transition-colors"
                :class="[
                  isRouteActive(item.path)
                    ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-semibold'
                    : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-white',
                ]"
                :aria-current="isRouteActive(item.path) ? 'page' : undefined"
              >
                <component
                  :is="item.icon"
                  class="w-4 h-4 shrink-0 transition-colors"
                  :class="[
                    isRouteActive(item.path)
                      ? 'text-indigo-600 dark:text-indigo-400'
                      : 'text-slate-400 dark:text-slate-500',
                  ]"
                />
                <span class="truncate">{{ item.name }}</span>
              </NuxtLink>
            </nav>
          </div>
        </div>


        <div class="p-3 border-t border-slate-200 dark:border-slate-800 text-center text-[11px] text-slate-400">
          <NuxtLink
            to="/dashboard"
            class="text-indigo-600 dark:text-indigo-400 hover:underline"
            @click="emit('close')"
          >
            Return to Dashboard
          </NuxtLink>
        </div>
      </div>
    </div>
  </Teleport>
</template>
