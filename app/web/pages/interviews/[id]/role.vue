<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceRole, importJobDescriptionUrl } = useInterviewService()

const { data: roleData, pending, error: roleError, refresh } = await useAsyncData(
  `role-spec-${workspaceId.value}`,
  () => getWorkspaceRole(workspaceId.value)
)

const importUrl = ref('')
const isImporting = ref(false)
const importError = ref('')
const importSuccess = ref('')

const handleImportUrl = async () => {
  if (!importUrl.value.trim()) return
  isImporting.value = true
  importError.value = ''
  importSuccess.value = ''
  try {
    await importJobDescriptionUrl(workspaceId.value, importUrl.value.trim())
    importSuccess.value = 'Job description imported successfully from URL.'
    importUrl.value = ''
    await refresh()
  } catch (err: unknown) {
    importError.value = err && typeof err === 'object' && 'message' in err && typeof err.message === 'string'
      ? err.message
      : 'Failed to import job description URL.'
  } finally {
    isImporting.value = false
  }
}
</script>

<template>
  <div class="space-y-6 max-w-4xl">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
        <span>ROLE SPECIFICATION</span>
        <span>•</span>
        <span class="capitalize">{{ roleData?.seniority || 'Staff' }} Level</span>
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ roleData?.title || 'Target Role Analysis' }}
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
        Structured decomposition of responsibilities and core technical competency requirements.
      </p>
    </div>


    <div class="p-4 bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-800 rounded-lg space-y-3 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="text-xs font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
          Import Job Posting from URL
        </div>
        <span class="text-[10px] font-mono text-slate-400">Greenhouse • Lever • Ashby • Workday • JSON-LD</span>
      </div>
      <div class="flex flex-col sm:flex-row gap-2">
        <input
          v-model="importUrl"
          type="url"
          placeholder="https://boards.greenhouse.io/company/jobs/12345"
          class="flex-1 text-xs px-3 py-2 rounded border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 font-mono"
          :disabled="isImporting"
        />
        <button
          type="button"
          class="px-4 py-2 text-xs font-semibold rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors shrink-0"
          :disabled="isImporting || !importUrl.trim()"
          @click="handleImportUrl"
        >
          {{ isImporting ? 'Extracting...' : 'Extract & Ingest' }}
        </button>
      </div>
      <div v-if="importSuccess" class="text-xs text-emerald-600 dark:text-emerald-400 font-medium">
        {{ importSuccess }}
      </div>
      <div v-if="importError" class="text-xs text-rose-600 dark:text-rose-400 font-medium">
        {{ importError }}
      </div>
    </div>

    <div v-if="roleError" class="p-4 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-xs space-y-2">
      <div class="font-semibold text-rose-800 dark:text-rose-300">Failed to load target role specification</div>
      <p class="text-rose-600 dark:text-rose-400">{{ roleError.message || String(roleError) }}</p>
      <button
        type="button"
        class="px-3 py-1 bg-rose-600 hover:bg-rose-700 text-white rounded font-medium transition-colors"
        @click="() => refresh()"
      >
        Retry Role Fetch
      </button>
    </div>

    <div v-else-if="pending" class="space-y-4">
      <div class="h-6 bg-slate-200 dark:bg-slate-800 rounded animate-pulse w-1/3"></div>
      <div class="h-20 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
      <div class="h-32 bg-slate-100 dark:bg-slate-800/40 rounded-lg animate-pulse"></div>
    </div>

    <template v-else>
      <div class="space-y-4">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
          Primary Architectural Responsibilities
        </h2>
        <div v-if="!roleData?.responsibilities?.length" class="p-6 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
          Extracting responsibilities from role specification...
        </div>
      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="(resp, idx) in roleData.responsibilities"
          :key="idx"
          class="p-4 flex items-start gap-3 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <span class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold shrink-0 mt-0.5">
            {{ String(idx + 1).padStart(2, '0') }}.
          </span>
          <p class="text-xs text-slate-700 dark:text-slate-300 m-0 leading-relaxed">
            {{ resp }}
          </p>
        </div>
      </div>
    </div>


    <div class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Competency & Domain Requirements
      </h2>
      <div v-if="!roleData?.requirements?.length" class="p-6 border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-xs text-slate-400">
        Extracting requirements from role specification...
      </div>
      <div v-else class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-sm">
        <div
          v-for="(req, idx) in roleData.requirements"
          :key="idx"
          class="p-4 space-y-1.5 hover:bg-slate-50 dark:hover:bg-slate-850 transition-colors"
        >
          <div class="flex items-center gap-2">
            <span
              class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded font-bold"
              :class="req.is_mandatory ? 'bg-rose-50 text-rose-700 dark:bg-rose-950 dark:text-rose-300' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'"
            >
              {{ req.is_mandatory ? 'Mandatory' : 'Preferred' }}
            </span>
            <span class="text-xs font-semibold text-slate-900 dark:text-white">{{ req.category }}</span>
          </div>
          <p class="text-xs text-slate-700 dark:text-slate-300 m-0 leading-relaxed">
            {{ req.description }}
          </p>
        </div>
      </div>
    </div>
  </template>
  </div>
</template>
