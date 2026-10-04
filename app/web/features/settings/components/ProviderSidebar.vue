<script setup lang="ts">
import type { LlmConfigItem } from '../models/settings.model'
import { resolveProviderMetadata } from '../utils/settings.util'

const props = defineProps<{
  providers: LlmConfigItem[]
  activeProvider: string
}>()

const emit = defineEmits<{
  (e: 'select', provider: string): void
}>()

const handleSelect = (providerId: string): void => {
  emit('select', providerId)
}

const getDisplayName = (providerId: string): string => {
  return resolveProviderMetadata(providerId).name
}
</script>

<template>
  <aside class="space-y-1.5 bg-white dark:bg-slate-900 p-3 rounded-lg border border-slate-200 dark:border-slate-800 shadow-sm w-full">
    <div class="px-2 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-400">
      Available Providers
    </div>

    <div v-if="props.providers.length === 0" class="p-3 text-xs text-slate-400 italic">
      No providers configured
    </div>

    <div class="flex md:flex-col overflow-x-auto md:overflow-x-visible scrollbar-none gap-1.5 pb-1 md:pb-0">
      <button
        v-for="prov in props.providers"
        :key="prov.provider"
        type="button"
        class="whitespace-nowrap md:whitespace-normal w-auto md:w-full text-left p-2.5 rounded-md text-xs transition flex items-center justify-between gap-3 shrink-0 md:shrink"
        :class="props.activeProvider === prov.provider
          ? 'bg-indigo-50 dark:bg-indigo-950/70 text-indigo-700 dark:text-indigo-300 font-bold border border-indigo-200/50 dark:border-indigo-800/40'
          : 'text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'"
        @click="handleSelect(prov.provider)"
      >
        <div class="flex items-center gap-2">
          <span>{{ getDisplayName(prov.provider) }}</span>
          <span
            v-if="prov.is_inherited"
            class="text-[9px] font-bold px-1 rounded bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200"
            title="Inherited from organization public sharing"
          >
            SHARED
          </span>
        </div>

        <div class="flex items-center gap-1.5">
          <span
            class="w-2 h-2 rounded-full shrink-0"
            :class="prov.has_key ? 'bg-emerald-500' : 'bg-slate-300 dark:bg-slate-700'"
            :title="prov.has_key ? 'Key configured' : 'No key configured'"
          ></span>
        </div>
      </button>
    </div>
  </aside>
</template>
