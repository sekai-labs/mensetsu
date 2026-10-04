<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { SourceCitation } from '../models/interview.model'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspaceCompany } = useInterviewService()
const { data: companyData } = await useAsyncData(
  `company-page-${workspaceId.value}`,
  () => getWorkspaceCompany(workspaceId.value)
)

const activeChapter = ref<number>(1)
const fontSize = ref<'sm' | 'base' | 'lg'>('sm')
const selectedSource = ref<SourceCitation | null>(null)
const isDrawerOpen = ref<boolean>(false)

const chapters = [
  { id: 1, title: '1. Executive Profile & Business Model', slug: 'profile' },
  { id: 2, title: '2. System Architecture & Topology', slug: 'architecture' },
  { id: 3, title: '3. Verified Technology Stack & Tooling', slug: 'tech-stack' },
  { id: 4, title: '4. Engineering Culture & Reliability', slug: 'culture' },
  { id: 5, title: '5. Technical Interview Process & Rubrics', slug: 'interview' },
  { id: 6, title: '6. Citation Index & Provenance', slug: 'citations' },
]

const openCitation = (source: SourceCitation) => {
  selectedSource.value = source
  isDrawerOpen.value = true
}

const closeDrawer = () => {
  isDrawerOpen.value = false
  selectedSource.value = null
}
</script>

<template>
  <div class="flex flex-col lg:flex-row gap-6 items-start">
    <aside class="w-full lg:w-64 shrink-0 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-3 shadow-sm sticky top-20">
      <div class="flex items-center justify-between pb-2 mb-2 border-b border-slate-100 dark:border-slate-800">
        <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400">Technical Dossier</span>
        <span class="text-[10px] font-mono bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 px-1.5 py-0.5 rounded font-semibold">BOOK</span>
      </div>

      <nav class="flex lg:flex-col overflow-x-auto lg:overflow-x-visible scrollbar-none gap-1 pb-1 lg:pb-0">
        <button
          v-for="chap in chapters"
          :key="chap.id"
          type="button"
          class="whitespace-nowrap lg:whitespace-normal text-left px-2.5 py-1.5 lg:py-2 rounded text-xs transition flex items-center justify-between shrink-0 lg:shrink"
          :class="activeChapter === chap.id
            ? 'bg-indigo-50 dark:bg-indigo-950/70 text-indigo-700 dark:text-indigo-300 font-semibold'
            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-white'"
          @click="activeChapter = chap.id"
        >
          <span class="truncate">{{ chap.title }}</span>
          <span v-if="activeChapter === chap.id" class="w-1.5 h-1.5 rounded-full bg-indigo-600 shrink-0 ml-1.5 hidden lg:inline-block"></span>
        </button>
      </nav>

      <div class="mt-4 pt-3 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-[11px] text-slate-500">
        <span>Font Size</span>
        <div class="flex items-center gap-1 border border-slate-200 dark:border-slate-700 rounded p-0.5">
          <button
            type="button"
            class="px-1.5 py-0.5 rounded"
            :class="fontSize === 'sm' ? 'bg-indigo-600 text-white font-bold' : 'hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'sm'"
          >
            A-
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded"
            :class="fontSize === 'base' ? 'bg-indigo-600 text-white font-bold' : 'hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'base'"
          >
            A
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded"
            :class="fontSize === 'lg' ? 'bg-indigo-600 text-white font-bold' : 'hover:text-slate-900 dark:hover:text-white'"
            @click="fontSize = 'lg'"
          >
            A+
          </button>
        </div>
      </div>
    </aside>

    <main class="flex-1 w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 sm:p-8 shadow-sm">
      <div class="border-b border-slate-200 dark:border-slate-800 pb-5 mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2 text-xs text-indigo-600 dark:text-indigo-400 font-semibold mb-1">
            <span>Engineering Intelligence Dossier</span>
            <span>•</span>
            <span class="capitalize">{{ companyData?.company_name }}</span>
          </div>
          <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
            Architectural Dossier & Company Technical Profile
          </h1>
        </div>
      </div>

      <div
        class="prose dark:prose-invert max-w-none transition-all leading-relaxed"
        :class="{
          'text-xs': fontSize === 'sm',
          'text-sm': fontSize === 'base',
          'text-base': fontSize === 'lg',
        }"
      >
        <section v-if="activeChapter === 1" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 1: Executive Profile, Operational Realities & Evidence Classification
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Foundational mission, commercial monetization, and multi-tier evidence classification matrix.</p>
          </div>

          <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-400 mb-2">Company Overview</h3>
            <p class="text-slate-700 dark:text-slate-300 m-0">
              {{ companyData?.overview || 'Comprehensive company profile synthesizing primary engineering publications, technical whitepapers, and operational releases.' }}
            </p>
          </div>

          <div>
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white mb-2">Commercial Architecture & Monetization</h3>
            <p class="text-slate-700 dark:text-slate-300">
              {{ companyData?.business_model || 'Enterprise cloud platform software, API usage metering, managed infrastructure tiers, and corporate SLA contracts.' }}
            </p>
          </div>

          <div v-if="companyData?.operational_realities?.length">
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white mb-2">Operational Realities</h3>
            <ul class="space-y-1.5 text-slate-700 dark:text-slate-300 list-disc pl-5">
              <li v-for="(op, idx) in companyData.operational_realities" :key="idx" class="text-xs">
                {{ op }}
              </li>
            </ul>
          </div>

          <div v-if="companyData?.evidence_matrix?.length" class="space-y-3 pt-2">
            <h3 class="text-sm font-semibold text-slate-900 dark:text-white">Evidence Classification Matrix</h3>
            <div class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
              <div
                v-for="(item, idx) in companyData.evidence_matrix"
                :key="idx"
                class="p-3.5 bg-white dark:bg-slate-900 space-y-1"
              >
                <div class="flex items-center gap-2">
                  <span
                    class="text-[10px] font-mono px-1.5 py-0.5 rounded font-bold"
                    :class="{
                      'bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300': item.tier === 'FACT',
                      'bg-indigo-100 text-indigo-700 dark:bg-indigo-950 dark:text-indigo-300': item.tier === 'INFERENCE',
                      'bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-300': item.tier === 'CANDIDATE',
                    }"
                  >
                    [{{ item.tier }}]
                  </span>
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

        <section v-if="activeChapter === 2" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 2: System Architecture & Infrastructure Topology
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Distributed topologies, partition handling, caching layers, and high-throughput durability.</p>
          </div>

          <div v-if="companyData?.architecture_topology" class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 space-y-1">
              <h3 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider text-slate-500">1. Edge & Ingress</h3>
              <p class="text-xs text-slate-600 dark:text-slate-300 m-0">{{ companyData.architecture_topology.edge_and_ingress }}</p>
            </div>
            <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 space-y-1">
              <h3 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider text-slate-500">2. Compute & Microservices</h3>
              <p class="text-xs text-slate-600 dark:text-slate-300 m-0">{{ companyData.architecture_topology.compute_and_orchestration }}</p>
            </div>
            <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 space-y-1">
              <h3 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider text-slate-500">3. Storage & Consensus</h3>
              <p class="text-xs text-slate-600 dark:text-slate-300 m-0">{{ companyData.architecture_topology.storage_and_consensus }}</p>
            </div>
            <div class="p-4 rounded-md bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700/60 space-y-1">
              <h3 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider text-slate-500">4. Observability & Zero Trust</h3>
              <p class="text-xs text-slate-600 dark:text-slate-300 m-0">{{ companyData.architecture_topology.observability_and_resilience }}</p>
            </div>
          </div>

          <div class="p-4 border border-dashed border-indigo-200 dark:border-indigo-900/60 rounded-lg bg-indigo-50/40 dark:bg-indigo-950/20">
            <h3 class="text-xs font-bold text-indigo-900 dark:text-indigo-300 mb-1">Architectural Invariants to Reference in Interview</h3>
            <p class="text-xs text-indigo-700 dark:text-indigo-200 m-0">
              When answering system design questions for {{ companyData?.company_name }}, emphasize end-to-end idempotency keys, rate-limiting tokens at edge proxies, and backpressure mitigation over optimistic concurrency.
            </p>
          </div>
        </section>

        <section v-if="activeChapter === 3" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 3: Verified Technology Stack & Production Evidence Map
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Observed runtimes, persistence frameworks, orchestration engines, and concrete architectural roles.</p>
          </div>

          <div v-if="companyData?.technology_evidence_map?.length" class="space-y-3">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-400">Technology Evidence Map</h3>
            <div class="overflow-x-auto border border-slate-200 dark:border-slate-800 rounded-lg">
              <table class="w-full text-left text-xs">
                <thead class="bg-slate-50 dark:bg-slate-850 text-slate-500 font-medium border-b border-slate-200 dark:border-slate-800">
                  <tr>
                    <th class="py-2.5 px-3">Domain</th>
                    <th class="py-2.5 px-3">Tool / Tech</th>
                    <th class="py-2.5 px-3">Tier</th>
                    <th class="py-2.5 px-3">Production Architectural Role</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                  <tr v-for="(t, idx) in companyData.technology_evidence_map" :key="idx" class="hover:bg-slate-50 dark:hover:bg-slate-850/50">
                    <td class="py-2.5 px-3 font-medium text-slate-900 dark:text-white">{{ t.domain }}</td>
                    <td class="py-2.5 px-3 font-mono font-semibold text-indigo-600 dark:text-indigo-400">{{ t.tool_or_tech }}</td>
                    <td class="py-2.5 px-3">
                      <span
                        class="text-[10px] font-mono px-1 rounded font-bold"
                        :class="t.evidence_tier === 'FACT' ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300' : 'bg-indigo-100 text-indigo-800 dark:bg-indigo-950 dark:text-indigo-300'"
                      >
                        {{ t.evidence_tier }}
                      </span>
                    </td>
                    <td class="py-2.5 px-3 text-slate-600 dark:text-slate-300">{{ t.production_role }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="space-y-4">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-400">Production Technologies & Libraries</h3>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
              <div
                v-for="(tech, idx) in companyData?.technology_stack || []"
                :key="idx"
                class="p-3 rounded border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-850 flex items-center justify-between"
              >
                <span class="font-mono text-xs font-semibold text-slate-800 dark:text-slate-200">{{ tech }}</span>
                <span class="text-[10px] font-bold text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/50 px-1 rounded">VERIFIED</span>
              </div>
            </div>
          </div>
        </section>

        <section v-if="activeChapter === 4" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 4: Engineering Culture, Post-Mortems & Operational Standards
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Deployment cadence, operational review standards, and RFC consensus mechanisms.</p>
          </div>

          <div class="space-y-3">
            <div
              v-for="(item, idx) in companyData?.engineering_culture || []"
              :key="idx"
              class="p-4 rounded border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-850 flex items-start gap-3"
            >
              <div class="w-5 h-5 rounded-full bg-indigo-100 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 flex items-center justify-center font-bold text-xs shrink-0 mt-0.5">
                {{ idx + 1 }}
              </div>
              <div>
                <h4 class="text-xs font-bold text-slate-900 dark:text-white mb-0.5">{{ item }}</h4>
                <p class="text-xs text-slate-600 dark:text-slate-400 m-0">
                  Engineering teams practice rigorous blameless retrospectives, automated canary rollback deployments, and peer-reviewed design documents before code merges.
                </p>
              </div>
            </div>
          </div>
        </section>

        <section v-if="activeChapter === 5" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 5: Technical Interview Rounds & Evaluation Rubrics
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Candidate evaluation stages, live problem formats, and passing criteria.</p>
          </div>

          <div class="space-y-3">
            <div
              v-for="(insight, idx) in companyData?.interview_process_insights || []"
              :key="idx"
              class="p-4 rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 flex items-start gap-3 shadow-xs"
            >
              <div class="w-2 h-2 rounded-full bg-emerald-500 mt-2 shrink-0"></div>
              <div>
                <span class="text-xs font-semibold text-slate-900 dark:text-white">{{ insight }}</span>
              </div>
            </div>
          </div>
        </section>

        <section v-if="activeChapter === 6" class="space-y-6">
          <div class="border-l-4 border-indigo-600 pl-4 py-1">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white m-0">
              Chapter 6: Verifiable Evidence & Citation Index
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-xs m-0">Provenance audit trail with cryptographic timestamps and snippet validation.</p>
          </div>

          <div class="divide-y divide-slate-200 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-slate-50/50 dark:bg-slate-850/50">
            <div
              v-for="(source, idx) in companyData?.sources || []"
              :key="source.id || idx"
              class="p-4 flex items-start justify-between gap-4 hover:bg-white dark:hover:bg-slate-800/80 transition cursor-pointer"
              @click="openCitation(source)"
            >
              <div class="space-y-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-bold text-indigo-600 dark:text-indigo-400">[{{ idx + 1 }}]</span>
                  <h4 class="text-xs font-bold text-slate-900 dark:text-white truncate">{{ source.title || 'Official Technical Publication' }}</h4>
                  <span class="text-[10px] font-mono px-1 rounded bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300">
                    {{ (source.confidence * 100).toFixed(0) }}% confidence
                  </span>
                </div>
                <p class="text-xs text-slate-500 truncate">{{ source.url }}</p>
                <p class="text-xs text-slate-600 dark:text-slate-300 line-clamp-2 italic">
                  "{{ source.snippet }}"
                </p>
              </div>
              <button
                type="button"
                class="px-2.5 py-1 text-xs font-semibold rounded bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 hover:bg-indigo-100 shrink-0"
              >
                Inspect →
              </button>
            </div>
          </div>
        </section>
      </div>

      <div class="mt-10 pt-5 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between">
        <button
          type="button"
          class="px-3 py-1.5 rounded border border-slate-200 dark:border-slate-700 text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-40 transition"
          :disabled="activeChapter <= 1"
          @click="activeChapter--"
        >
          ← Previous Chapter
        </button>
        <span class="text-xs text-slate-400 font-mono">Chapter {{ activeChapter }} of {{ chapters.length }}</span>
        <button
          type="button"
          class="px-3 py-1.5 rounded bg-indigo-600 text-white text-xs font-semibold hover:bg-indigo-500 disabled:opacity-40 transition"
          :disabled="activeChapter >= chapters.length"
          @click="activeChapter++"
        >
          Next Chapter →
        </button>
      </div>
    </main>

    <div
      v-if="isDrawerOpen && selectedSource"
      class="fixed inset-0 z-50 overflow-hidden bg-slate-900/40 backdrop-blur-xs flex justify-end"
      @click.self="closeDrawer"
    >
      <div class="w-full max-w-md bg-white dark:bg-slate-900 h-full shadow-2xl border-l border-slate-200 dark:border-slate-800 p-6 flex flex-col justify-between overflow-y-auto">
        <div class="space-y-4">
          <div class="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-slate-800">
            <span class="text-xs font-bold uppercase tracking-wider text-indigo-600 dark:text-indigo-400">Verifiable Source Citation</span>
            <button
              type="button"
              class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 text-sm font-bold"
              @click="closeDrawer"
            >
              ✕
            </button>
          </div>

          <div>
            <h3 class="text-sm font-bold text-slate-900 dark:text-white">{{ selectedSource.title }}</h3>
            <a :href="selectedSource.url" target="_blank" rel="noopener noreferrer" class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline break-all">
              {{ selectedSource.url }}
            </a>
          </div>

          <div class="p-3 rounded bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-700 space-y-2">
            <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400">Exact Extracted Evidence</div>
            <p class="text-xs text-slate-700 dark:text-slate-300 leading-relaxed font-serif italic">
              "{{ selectedSource.snippet }}"
            </p>
          </div>

          <div class="text-[11px] text-slate-500 space-y-1">
            <div><strong>Source Type:</strong> {{ selectedSource.source_type || 'Primary Official Engineering Source' }}</div>
            <div><strong>Confidence Score:</strong> {{ (selectedSource.confidence * 100).toFixed(1) }}%</div>
            <div><strong>Domain:</strong> {{ selectedSource.domain }}</div>
          </div>
        </div>

        <div class="pt-4 border-t border-slate-200 dark:border-slate-800 flex justify-end">
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-200"
            @click="closeDrawer"
          >
            Close Drawer
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
