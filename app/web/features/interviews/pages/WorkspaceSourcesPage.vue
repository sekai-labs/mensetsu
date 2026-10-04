<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import { formatDateTime } from '~/utils/formatters'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceCompany } = useInterviewService()
const { data: companyData } = await useAsyncData(
  `sources-page-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
      <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
        Research Provenance & Verified Sources
      </h1>
      <p class="text-xs text-slate-500 mt-1">
        Citations and timestamped extracts gathered autonomously during background research.
      </p>
    </div>

    <div
      v-if="!companyData?.sources || companyData.sources.length === 0"
      class="p-8 text-center text-xs text-slate-500 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg"
    >
      No external web citations logged yet.
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="source in companyData.sources"
        :key="source.id"
        class="p-5 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-3"
      >
        <div class="flex items-center justify-between text-xs">
          <span class="font-mono text-slate-500">{{ source.domain }}</span>
          <span class="font-mono text-emerald-600 font-semibold">
            {{ Math.round(source.confidence * 100) }}% confidence
          </span>
        </div>

        <h3 class="text-sm font-semibold text-slate-900 dark:text-white">
          <a
            :href="source.url"
            target="_blank"
            rel="noopener noreferrer"
            class="hover:text-indigo-600 transition flex items-center gap-1.5"
          >
            {{ source.title }} <span class="text-xs text-slate-400">↗</span>
          </a>
        </h3>

        <blockquote
          class="text-xs text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-850 p-3.5 rounded border border-slate-100 dark:border-slate-800 font-sans italic leading-relaxed"
        >
          "{{ source.snippet }}"
        </blockquote>

        <div class="text-[11px] text-slate-400 font-mono">
          Retrieved: {{ formatDateTime(source.retrieved_at) }}
        </div>
      </div>
    </div>
  </div>
</template>
