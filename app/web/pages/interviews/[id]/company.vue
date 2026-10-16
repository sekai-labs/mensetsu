<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { SourceCitation } from '~/features/interviews/models/interview.model'
import EvidenceBadge from '~/components/ui/EvidenceBadge.vue'
import EvidenceDrawer from '~/components/ui/EvidenceDrawer.vue'
import MarkdownRenderer from '~/components/ui/MarkdownRenderer.vue'
definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceCompany } = useInterviewService()

const { data: companyData } = await useAsyncData(
  `company-dossier-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)

const activeChapterId = ref<string>('profile')
const fontSize = ref<'sm' | 'base' | 'lg'>('base')
const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)

const defaultChapters = [
  { id: 'profile', title: '1. Executive Profile & Evidence Matrix', slug: 'profile' },
  { id: 'architecture', title: '2. System Architecture & Topology', slug: 'architecture' },
  { id: 'tech-stack', title: '3. Verified Technology Stack', slug: 'tech-stack' },
  { id: 'culture', title: '4. Engineering Culture & Reliability', slug: 'culture' },
  { id: 'interview', title: '5. Interview Process & Rubrics', slug: 'interview' },
  { id: 'citations', title: '6. Citation Index & Provenance', slug: 'citations' },
]

const displayedChapters = computed(() => {
  const custom = companyData.value?.chapters || []
  if (custom.length > 0) {
    const list = custom.map((chap, idx) => ({
      id: chap.id || `chapter_${idx + 1}`,
      title: `${idx + 1}. ${chap.title}`,
      slug: chap.id || `chapter_${idx + 1}`,
      custom: true,
      summary: chap.summary,
      content_markdown: chap.content_markdown,
    }))
    // Add verified tech stack & citations as companion references if not in custom list
    if (companyData.value?.tech_stack?.length && !list.some((c) => c.slug === 'tech-stack')) {
      list.push({
        id: 'tech-stack',
        title: `${list.length + 1}. Verified Technology Stack`,
        slug: 'tech-stack',
        custom: false,
        summary: 'Production runtime technologies with citations.',
        content_markdown: undefined,
      })
    }
    if (companyData.value?.sources?.length && !list.some((c) => c.slug === 'citations')) {
      list.push({
        id: 'citations',
        title: `${list.length + 1}. Citation Index & Provenance`,
        slug: 'citations',
        custom: false,
        summary: 'Exhaustive index of indexed engineering sources and retrieval confidence scores.',
        content_markdown: undefined,
      })
    }
    return list
  }
  return defaultChapters.map((c) => ({ ...c, custom: false, summary: undefined, content_markdown: undefined }))
})

// Keep activeChapterId pointing to first available chapter if unset
watchEffect(() => {
  if (displayedChapters.value.length > 0 && !displayedChapters.value.some((c) => c.id === activeChapterId.value)) {
    activeChapterId.value = displayedChapters.value[0].id
  }
})

const activeCustomChapter = computed(() => {
  return displayedChapters.value.find((c) => c.id === activeChapterId.value && c.custom)
})

const openCitation = (source: SourceCitation) => {
  selectedSource.value = source
  isDrawerOpen.value = true
}

const openSourceById = (sourceId?: string) => {
  if (!sourceId || !companyData.value?.sources) return
  const found = companyData.value.sources.find((s) => s.id === sourceId)
  if (found) {
    openCitation(found)
  }
}
</script>
<template>
  <div class="space-y-6">

    <div class="border-b border-slate-200 dark:border-slate-800 pb-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2 text-xs font-mono text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
          <span>{{ $t('company.badge') }}</span>
          <span>•</span>
          <span class="capitalize">{{ companyData?.company_name || 'Target Organization' }}</span>
        </div>
        <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
          {{ $t('company.title') }}
        </h1>
      </div>


      <div class="flex items-center gap-3">
        <span class="text-xs text-slate-400 font-mono">{{ $t('company.fontSize') }}:</span>
        <div class="flex items-center gap-1 border border-slate-200 dark:border-slate-700 rounded p-0.5 bg-white dark:bg-slate-900">
          <button
            type="button"
            class="px-2 py-0.5 text-xs rounded transition-colors"
            :class="fontSize === 'sm' ? 'bg-indigo-600 text-white font-bold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'sm'"
          >
            A-
          </button>
          <button
            type="button"
            class="px-2 py-0.5 text-xs rounded transition-colors"
            :class="fontSize === 'base' ? 'bg-indigo-600 text-white font-bold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'base'"
          >
            A
          </button>
          <button
            type="button"
            class="px-2 py-0.5 text-xs rounded transition-colors"
            :class="fontSize === 'lg' ? 'bg-indigo-600 text-white font-bold' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'lg'"
          >
            A+
          </button>
        </div>
      </div>
    </div>


    <div class="flex flex-col lg:flex-row gap-8 items-start">

      <aside class="w-full lg:w-60 shrink-0 sticky top-20 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-3 shadow-sm">
        <div class="px-2 py-1 text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400 border-b border-slate-100 dark:border-slate-800 mb-2">
          Document Outline
        </div>
        <nav class="flex lg:flex-col overflow-x-auto lg:overflow-x-visible scrollbar-none gap-1 pb-1 lg:pb-0">
          <button
            v-for="chap in displayedChapters"
            :key="chap.id"
            type="button"
            class="whitespace-nowrap lg:whitespace-normal text-left px-2.5 py-1.5 rounded text-xs transition-colors shrink-0 lg:shrink flex items-center justify-between"
            :class="activeChapterId === chap.id
              ? 'bg-indigo-50 dark:bg-indigo-950/70 text-indigo-700 dark:text-indigo-300 font-semibold'
              : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-slate-900 dark:hover:text-white'"
            @click="activeChapterId = chap.id"
          >
            <span class="truncate">{{ chap.title }}</span>
            <span v-if="activeChapterId === chap.id" class="w-1.5 h-1.5 rounded-full bg-indigo-600 shrink-0 ml-1.5 hidden lg:inline-block"></span>
          </button>
        </nav>
      </aside>

      <article
        class="flex-1 w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 sm:p-8 shadow-sm transition-all"
        :class="{
          'text-xs leading-relaxed': fontSize === 'sm',
          'text-sm leading-relaxed': fontSize === 'base',
          'text-base leading-relaxed': fontSize === 'lg',
        }"
      >

        <!-- Custom Agent-Decided Chapter -->
        <section v-if="activeCustomChapter" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              {{ activeCustomChapter.title }}
            </h2>
            <p v-if="activeCustomChapter.summary" class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              {{ activeCustomChapter.summary }}
            </p>
          </div>

          <div v-if="activeCustomChapter.content_markdown" class="prose prose-slate dark:prose-invert max-w-none text-slate-700 dark:text-slate-300">
            <MarkdownRenderer :content="activeCustomChapter.content_markdown" />
          </div>
          <div v-else class="text-xs text-slate-400 italic">
            No content recorded for this chapter.
          </div>
        </section>

        <!-- Standard Chapter: Executive Profile & Evidence Matrix -->
        <section v-else-if="activeChapterId === 'profile' || activeChapterId === '1'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 1: Executive Profile & Evidence Classification
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Foundational mission, commercial monetization, and multi-tier evidence classification matrix.
            </p>
          </div>

          <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60">
            <h3 class="text-xs font-mono font-bold uppercase tracking-wider text-slate-500 mb-2">Company Overview</h3>
            <p v-if="companyData?.overview" class="text-slate-700 dark:text-slate-300">
              {{ companyData.overview }}
            </p>
            <p v-else class="text-slate-400 italic text-xs">
              No overview extracted for this organization yet.
            </p>
          </div>

          <div class="space-y-2">
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Commercial Architecture & Monetization</h3>
            <p v-if="companyData?.business_model" class="text-slate-700 dark:text-slate-300">
              {{ companyData.business_model }}
            </p>
            <p v-else class="text-slate-400 italic text-xs">
              No business model extracted for this organization yet.
            </p>
          </div>
          <div v-if="companyData?.operational_realities?.length" class="space-y-2">
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Operational Realities</h3>
            <ul class="space-y-1.5 list-disc pl-5 text-slate-700 dark:text-slate-300">
              <li v-for="(op, idx) in companyData.operational_realities" :key="idx">
                {{ op }}
              </li>
            </ul>
          </div>


          <div v-if="companyData?.evidence_matrix?.length" class="space-y-3 pt-3">
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Evidence Classification Matrix</h3>
            <div class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
              <div
                v-for="(item, idx) in companyData.evidence_matrix"
                :key="idx"
                class="p-3.5 bg-white dark:bg-slate-900 space-y-1.5"
              >
                <div class="flex items-center gap-2">
                  <EvidenceBadge :tier="item.tier" />
                  <span class="text-xs font-semibold text-slate-900 dark:text-white">{{ item.domain }}</span>
                </div>
                <p class="text-xs text-slate-700 dark:text-slate-300 m-0">{{ item.statement }}</p>
                <p v-if="item.architectural_role" class="text-[11px] text-slate-500 m-0">
                  <span class="font-medium text-slate-600 dark:text-slate-400">Architectural Role:</span> {{ item.architectural_role }}
                </p>
              </div>
            </div>
          </div>
        </section>


        <section v-else-if="activeChapterId === 'architecture' || activeChapterId === '2'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 2: System Architecture & Infrastructure Topology
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Distributed topologies, partition handling, caching layers, and high-throughput durability.
            </p>
          </div>

          <div v-if="companyData?.architecture_topology" class="space-y-4">
            <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 space-y-2">
              <div class="text-xs font-mono font-bold uppercase tracking-wider text-slate-500">System Architecture Type</div>
              <div class="text-sm font-semibold text-slate-900 dark:text-white">
                {{ companyData.architecture_topology.system_type }}
              </div>
              <div v-if="companyData.architecture_topology.consensus_or_sync_mechanism" class="text-xs text-slate-600 dark:text-slate-300">
                <span class="font-medium text-slate-700 dark:text-slate-200">Consensus / Sync:</span> {{ companyData.architecture_topology.consensus_or_sync_mechanism }}
              </div>
            </div>

            <div class="space-y-2">
              <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Core System Components</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="(comp, idx) in companyData.architecture_topology.components"
                  :key="idx"
                  class="px-2.5 py-1 rounded bg-slate-100 dark:bg-slate-800 text-xs font-mono text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700"
                >
                  {{ comp }}
                </span>
              </div>
            </div>

            <div class="space-y-2">
              <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Data Storage & Durability Layers</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="(store, idx) in companyData.architecture_topology.data_stores"
                  :key="idx"
                  class="px-2.5 py-1 rounded bg-indigo-50 dark:bg-indigo-950/60 text-xs font-mono text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800"
                >
                  {{ store }}
                </span>
              </div>
            </div>
          </div>
          <div v-else class="text-xs text-slate-500">
            Architecture topology extraction in progress.
          </div>
        </section>


        <section v-else-if="activeChapterId === 'tech-stack' || activeChapterId === '3'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 3: Verified Technology Stack & Tooling
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Production runtime technologies with citations to engineering posts and public repositories.
            </p>
          </div>

          <div v-if="companyData?.tech_stack?.length" class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
            <div
              v-for="(tech, idx) in companyData.tech_stack"
              :key="idx"
              class="p-3.5 bg-white dark:bg-slate-900 flex items-center justify-between gap-4"
            >
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold text-slate-900 dark:text-white font-mono">{{ tech.name }}</span>
                  <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">
                    {{ tech.category }}
                  </span>
                </div>
                <p v-if="tech.notes" class="text-xs text-slate-500 m-0">{{ tech.notes }}</p>
              </div>

              <div class="shrink-0 flex items-center gap-2">
                <span
                  class="text-[10px] font-mono px-2 py-0.5 rounded font-semibold"
                  :class="tech.verified ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'"
                >
                  {{ tech.verified ? 'VERIFIED' : 'UNVERIFIED' }}
                </span>
                <button
                  v-if="tech.source_citation_id"
                  type="button"
                  class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium"
                  @click="openSourceById(tech.source_citation_id)"
                >
                  Provenance →
                </button>
              </div>
            </div>
          </div>
        </section>


        <section v-else-if="activeChapterId === 'culture' || activeChapterId === '4'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 4: Engineering Culture, Cadence & Reliability
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Code review standards, incident response philosophy, and testing requirements.
            </p>
          </div>

          <div v-if="companyData?.engineering_culture?.length" class="space-y-3">
            <div
              v-for="(cult, idx) in companyData.engineering_culture"
              :key="idx"
              class="p-3.5 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-800 text-xs text-slate-700 dark:text-slate-300 leading-relaxed"
            >
              {{ cult }}
            </div>
          </div>
        </section>


        <section v-else-if="activeChapterId === 'interview' || activeChapterId === '5'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 5: Technical Interview Process & Rubrics
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Reported loop stages, system design focus, coding bar, and interviewer expectations.
            </p>
          </div>

          <div v-if="companyData?.interview_process_notes?.length" class="space-y-3">
            <div
              v-for="(note, idx) in companyData.interview_process_notes"
              :key="idx"
              class="p-3.5 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-800 text-xs text-slate-700 dark:text-slate-300 leading-relaxed"
            >
              {{ note }}
            </div>
          </div>
        </section>


        <section v-else-if="activeChapterId === 'citations' || activeChapterId === '6'" class="space-y-6 max-w-prose">
          <div class="border-l-3 border-indigo-600 pl-4">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              Chapter 6: Citation Index & Provenance
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              Exhaustive index of indexed engineering sources, snippets, and retrieval confidence scores.
            </p>
          </div>

          <div v-if="companyData?.sources?.length" class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
            <div
              v-for="src in companyData.sources"
              :key="src.id"
              class="p-4 bg-white dark:bg-slate-900 space-y-2 hover:bg-slate-50 dark:hover:bg-slate-850 cursor-pointer transition-colors"
              @click="openCitation(src)"
            >
              <div class="flex items-center justify-between gap-4">
                <span class="text-xs font-semibold text-slate-900 dark:text-white">{{ src.title }}</span>
                <span class="text-[10px] font-mono text-emerald-600 dark:text-emerald-400 font-bold">
                  {{ Math.round((src.confidence || 1) * 100) }}% CONFIDENCE
                </span>
              </div>
              <p class="text-xs text-slate-600 dark:text-slate-400 line-clamp-2 italic">
                "{{ src.snippet }}"
              </p>
              <div class="text-[11px] font-mono text-indigo-600 dark:text-indigo-400 truncate">
                {{ src.source_url }}
              </div>
            </div>
          </div>
        </section>
      </article>
    </div>


    <EvidenceDrawer
      :is-open="isDrawerOpen"
      :citation="selectedSource"
      @close="isDrawerOpen = false"
    />
  </div>
</template>
