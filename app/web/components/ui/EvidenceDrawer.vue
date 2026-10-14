<script setup lang="ts">
import type { SourceCitation } from '~/features/interviews/models/interview.model'

interface Props {
  isOpen: boolean
  citation: SourceCitation | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.isOpen) {
    emit('close')
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 overflow-hidden bg-slate-950/60 transition-opacity"
      role="dialog"
      aria-modal="true"
      aria-labelledby="drawer-title"
      @click.self="emit('close')"
    >
      <div class="fixed inset-y-0 right-0 max-w-full flex pl-10">
        <div class="w-screen max-w-md bg-white dark:bg-slate-900 border-l border-slate-200 dark:border-slate-800 shadow-xl flex flex-col">
          <div class="px-5 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
              <h2 id="drawer-title" class="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                Verified Provenance Citation
              </h2>
            </div>
            <button
              type="button"
              class="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors"
              aria-label="Close panel"
              @click="emit('close')"
            >
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div v-if="citation" class="flex-1 overflow-y-auto p-5 space-y-5">
            <div>
              <span class="text-[11px] font-mono text-slate-400 block mb-1">Source Title</span>
              <h3 class="text-sm font-semibold text-slate-900 dark:text-white leading-snug">
                {{ citation.title }}
              </h3>
            </div>

            <div class="flex items-center gap-4 text-xs font-mono">
              <div>
                <span class="text-slate-400 block text-[10px]">Confidence</span>
                <span class="text-emerald-600 dark:text-emerald-400 font-bold">
                  {{ Math.round((citation.confidence || 1) * 100) }}%
                </span>
              </div>
              <div>
                <span class="text-slate-400 block text-[10px]">Retrieved</span>
                <span class="text-slate-700 dark:text-slate-300">
                  {{ citation.retrieved_at ? new Date(citation.retrieved_at).toLocaleDateString() : 'Recent' }}
                </span>
              </div>
            </div>

            <div>
              <span class="text-[11px] font-mono text-slate-400 block mb-1.5">Extracted Evidence Snippet</span>
              <blockquote class="p-3.5 rounded bg-slate-50 dark:bg-slate-850 border-l-2 border-indigo-500 text-xs text-slate-700 dark:text-slate-300 italic leading-relaxed">
                "{{ citation.snippet }}"
              </blockquote>
            </div>

            <div>
              <span class="text-[11px] font-mono text-slate-400 block mb-1.5">Original Source Document</span>
              <a
                :href="citation.source_url"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-1.5 text-xs font-mono text-indigo-600 dark:text-indigo-400 hover:underline break-all"
              >
                <span>{{ citation.source_url }}</span>
                <svg class="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </a>
            </div>
          </div>

          <div class="p-4 border-t border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/50 flex justify-end">
            <button
              type="button"
              class="px-3 py-1.5 text-xs font-medium rounded-md border border-slate-300 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 transition-colors"
              @click="emit('close')"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
