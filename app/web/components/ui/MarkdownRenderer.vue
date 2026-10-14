<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '~/features/interviews/utils/markdown.util'

interface Props {
  content?: string | null
  dense?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  content: '',
  dense: false,
})

const renderedHtml = computed(() => renderMarkdown(props.content))
</script>

<template>
  <div
    class="markdown-content font-sans text-slate-800 dark:text-slate-200"
    :class="{ 'dense-markdown': dense }"
    v-html="renderedHtml"
  />
</template>

<style scoped>
.markdown-content :deep(h1) {
  @apply text-xl font-bold tracking-tight text-slate-900 dark:text-white mt-6 mb-3;
}
.markdown-content :deep(h2) {
  @apply text-lg font-bold tracking-tight text-slate-900 dark:text-white mt-5 mb-2.5 border-b border-slate-200 dark:border-slate-800 pb-1.5;
}
.markdown-content :deep(h3) {
  @apply text-sm font-semibold text-slate-900 dark:text-white mt-4 mb-2;
}
.markdown-content :deep(h4),
.markdown-content :deep(h5),
.markdown-content :deep(h6) {
  @apply text-xs font-semibold text-slate-800 dark:text-slate-200 mt-3 mb-1.5 uppercase tracking-wider font-mono;
}
.markdown-content :deep(p) {
  @apply text-xs sm:text-sm leading-relaxed mb-3.5 text-slate-700 dark:text-slate-300;
}
.markdown-content :deep(ul) {
  @apply list-disc list-outside pl-5 mb-3.5 space-y-1 text-xs sm:text-sm text-slate-700 dark:text-slate-300;
}
.markdown-content :deep(ol) {
  @apply list-decimal list-outside pl-5 mb-3.5 space-y-1 text-xs sm:text-sm text-slate-700 dark:text-slate-300;
}
.markdown-content :deep(li) {
  @apply leading-relaxed;
}
.markdown-content :deep(blockquote) {
  @apply border-l-2 border-indigo-500 pl-3.5 py-1 my-3 bg-slate-50 dark:bg-slate-850/60 rounded-r text-xs sm:text-sm italic text-slate-600 dark:text-slate-400;
}
.markdown-content :deep(pre) {
  @apply my-3.5 p-3.5 rounded-lg bg-slate-900 dark:bg-slate-950 text-slate-100 font-mono text-xs overflow-x-auto border border-slate-800 shadow-sm;
}
.markdown-content :deep(code) {
  @apply font-mono text-xs px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-indigo-700 dark:text-indigo-300 border border-slate-200 dark:border-slate-700;
}
.markdown-content :deep(pre code) {
  @apply p-0 bg-transparent text-slate-100 border-none;
}
.markdown-content :deep(table) {
  @apply w-full text-left border-collapse my-3.5 text-xs;
}
.markdown-content :deep(th) {
  @apply border-b border-slate-300 dark:border-slate-700 font-semibold p-2 bg-slate-50 dark:bg-slate-850 text-slate-900 dark:text-white;
}
.markdown-content :deep(td) {
  @apply border-b border-slate-200 dark:border-slate-800 p-2 text-slate-700 dark:text-slate-300;
}
.markdown-content :deep(hr) {
  @apply my-6 border-t border-slate-200 dark:border-slate-800;
}
.markdown-content :deep(a) {
  @apply text-indigo-600 dark:text-indigo-400 underline font-medium hover:text-indigo-500;
}
.markdown-content :deep(strong) {
  @apply font-semibold text-slate-900 dark:text-white;
}

/* Dense mode for compact cards, syllabus previews, summaries */
.dense-markdown :deep(h1),
.dense-markdown :deep(h2),
.dense-markdown :deep(h3) {
  @apply text-xs font-semibold mt-2 mb-1;
}
.dense-markdown :deep(p) {
  @apply text-xs leading-normal mb-1.5;
}
.dense-markdown :deep(ul),
.dense-markdown :deep(ol) {
  @apply pl-4 mb-1.5 space-y-0.5 text-xs;
}
</style>
