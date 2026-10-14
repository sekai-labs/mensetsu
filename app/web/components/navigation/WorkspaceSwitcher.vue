<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { onClickOutside } from '@vueuse/core'
import {
  Building2,
  Check,
  ChevronDown,
  Plus,
  Search,
} from 'lucide-vue-next'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { WorkspaceResponse } from '~/features/interviews/models/interview.model'
import StatusBadge from '~/components/ui/StatusBadge.vue'

interface Props {
  currentWorkspaceId?: string
  currentWorkspace?: WorkspaceResponse | null
}

const props = withDefaults(defineProps<Props>(), {
  currentWorkspaceId: '',
  currentWorkspace: null,
})

const router = useRouter()
const { listWorkspaces, cleanWorkspaceId } = useInterviewService()

const isOpen = ref(false)
const searchQuery = ref('')
const workspaces = ref<WorkspaceResponse[]>([])
const isLoading = ref(false)
const selectedIndex = ref(-1)

const containerRef = ref<HTMLElement | null>(null)
const searchInputRef = ref<HTMLInputElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)

const loadWorkspaces = async () => {
  try {
    isLoading.value = true
    const list = await listWorkspaces()
    workspaces.value = Array.isArray(list) ? list : []
  } catch {

    if (props.currentWorkspace) {
      workspaces.value = [props.currentWorkspace]
    }
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  loadWorkspaces()
})

const currentDisplayCompany = computed(() => {
  if (props.currentWorkspace?.company_name) {
    return props.currentWorkspace.company_name
  }
  const match = workspaces.value.find(
    (w) => cleanWorkspaceId(w.id) === cleanWorkspaceId(props.currentWorkspaceId)
  )
  return match?.company_name || 'Select Workspace'
})

const currentDisplayRole = computed(() => {
  if (props.currentWorkspace?.role_title) {
    return props.currentWorkspace.role_title
  }
  const match = workspaces.value.find(
    (w) => cleanWorkspaceId(w.id) === cleanWorkspaceId(props.currentWorkspaceId)
  )
  return match?.role_title || ''
})


const filteredWorkspaces = computed(() => {
  if (!searchQuery.value.trim()) return workspaces.value
  const q = searchQuery.value.toLowerCase().trim()
  return workspaces.value.filter(
    (w) =>
      w.company_name?.toLowerCase().includes(q) ||
      w.role_title?.toLowerCase().includes(q)
  )
})

const toggleOpen = () => {
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    searchQuery.value = ''
    selectedIndex.value = -1

    loadWorkspaces()
    setTimeout(() => {
      searchInputRef.value?.focus()
    }, 50)
  }
}

const close = () => {
  if (isOpen.value) {
    isOpen.value = false
    triggerRef.value?.focus()
  }
}

onClickOutside(containerRef, () => {
  close()
})

const selectWorkspace = (ws: WorkspaceResponse) => {
  const cleanId = cleanWorkspaceId(ws.id)
  isOpen.value = false
  router.push(`/interviews/${cleanId}/overview`)
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) return

  if (e.key === 'Escape') {
    e.preventDefault()
    close()
    return
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (selectedIndex.value < filteredWorkspaces.value.length - 1) {
      selectedIndex.value++
    }
    return
  }

  if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (selectedIndex.value > 0) {
      selectedIndex.value--
    }
    return
  }

  if (e.key === 'Enter') {
    e.preventDefault()
    if (
      selectedIndex.value >= 0 &&
      selectedIndex.value < filteredWorkspaces.value.length
    ) {
      selectWorkspace(filteredWorkspaces.value[selectedIndex.value])
    }
    return
  }
}
</script>

<template>
  <div ref="containerRef" class="relative inline-block text-left" @keydown="handleKeydown">

    <button
      ref="triggerRef"
      type="button"
      class="group flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50/80 dark:bg-slate-900/80 hover:bg-white dark:hover:bg-slate-800/90 text-left transition-all max-w-[220px] sm:max-w-[280px] focus:outline-none focus:ring-2 focus:ring-indigo-500"
      :class="isOpen ? 'ring-2 ring-indigo-500/30 border-indigo-300 dark:border-indigo-700' : ''"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      aria-label="Switch interview workspace"
      @click="toggleOpen"
    >
      <div class="w-6 h-6 rounded bg-slate-200 dark:bg-slate-800 flex items-center justify-center text-slate-700 dark:text-slate-300 shrink-0">
        <Building2 class="w-3.5 h-3.5" />
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-1.5">
          <span
            class="text-xs font-semibold text-slate-900 dark:text-white truncate"
            :title="currentDisplayCompany"
          >
            {{ currentDisplayCompany }}
          </span>
        </div>
        <div
          v-if="currentDisplayRole"
          class="text-[11px] text-slate-500 dark:text-slate-400 truncate"
          :title="currentDisplayRole"
        >
          {{ currentDisplayRole }}
        </div>
      </div>

      <ChevronDown
        class="w-3.5 h-3.5 text-slate-400 shrink-0 transition-transform duration-150"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>


    <div
      v-if="isOpen"
      class="absolute left-0 mt-2 w-80 rounded-lg bg-white dark:bg-slate-900 shadow-xl border border-slate-200 dark:border-slate-800 py-1.5 z-50 text-left animate-in fade-in zoom-in-95 duration-100"
      role="listbox"
      aria-label="Workspaces"
    >

      <div class="px-2.5 pb-2 pt-1 border-b border-slate-100 dark:border-slate-800">
        <div class="relative">
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-400" />
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            placeholder="Search target company or role..."
            class="w-full pl-8 pr-3 py-1.5 text-xs rounded-md bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </div>
      </div>


      <div class="max-h-60 overflow-y-auto py-1 scrollbar-thin">
        <div
          v-if="isLoading && workspaces.length === 0"
          class="px-4 py-3 text-center text-xs text-slate-400 font-mono"
        >
          Loading workspaces...
        </div>

        <div
          v-else-if="filteredWorkspaces.length === 0"
          class="px-4 py-4 text-center text-xs text-slate-500 dark:text-slate-400"
        >
          No workspaces match "{{ searchQuery }}"
        </div>

        <button
          v-for="(ws, idx) in filteredWorkspaces"
          :key="ws.id"
          type="button"
          class="w-full px-3 py-2 text-left flex items-center justify-between text-xs transition-colors hover:bg-slate-100 dark:hover:bg-slate-800/70"
          :class="[
            cleanWorkspaceId(ws.id) === cleanWorkspaceId(currentWorkspaceId)
              ? 'bg-indigo-50/70 dark:bg-indigo-950/40 text-indigo-900 dark:text-indigo-200 font-medium'
              : 'text-slate-700 dark:text-slate-300',
            selectedIndex === idx ? 'bg-slate-100 dark:bg-slate-800' : '',
          ]"
          role="option"
          :aria-selected="cleanWorkspaceId(ws.id) === cleanWorkspaceId(currentWorkspaceId)"
          @click="selectWorkspace(ws)"
          @mouseenter="selectedIndex = idx"
        >
          <div class="min-w-0 pr-2">
            <div class="flex items-center gap-1.5 font-medium truncate">
              <span class="truncate">{{ ws.company_name }}</span>
              <span
                v-if="ws.seniority"
                class="text-[10px] uppercase font-mono px-1 py-0.2 rounded bg-slate-100 dark:bg-slate-800 text-slate-500"
              >
                {{ ws.seniority }}
              </span>
            </div>
            <div class="text-[11px] text-slate-500 dark:text-slate-400 truncate">
              {{ ws.role_title }}
            </div>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <StatusBadge :status="ws.status" size="xs" :show-dot="true" />
            <Check
              v-if="cleanWorkspaceId(ws.id) === cleanWorkspaceId(currentWorkspaceId)"
              class="w-4 h-4 text-indigo-600 dark:text-indigo-400 ml-1"
            />
          </div>
        </button>
      </div>


      <div class="pt-1.5 mt-1 border-t border-slate-100 dark:border-slate-800 px-2">
        <NuxtLink
          to="/interviews/new"
          class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 transition-colors"
          @click="isOpen = false"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>New Target Workspace</span>
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
