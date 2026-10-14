<script setup lang="ts">
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'default',
})
</script>

<template>
  <div class="py-12 sm:py-20 max-w-5xl mx-auto space-y-20">

    <section class="text-center space-y-6 max-w-3xl mx-auto">
      <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-indigo-200 dark:border-indigo-900 bg-indigo-50/60 dark:bg-indigo-950/40 text-xs font-mono font-medium text-indigo-700 dark:text-indigo-300">
        <span class="w-1.5 h-1.5 rounded-full bg-indigo-600 dark:bg-indigo-400"></span>
        <span>EVIDENCE-BACKED INTERVIEW WORKSPACE</span>
      </div>

      <h1 class="text-4xl sm:text-5xl font-bold tracking-tight text-slate-900 dark:text-white leading-[1.15]">
        Rigorous engineering preparation grounded in verified truth.
      </h1>

      <p class="text-base sm:text-lg text-slate-600 dark:text-slate-400 leading-relaxed max-w-2xl mx-auto">
        Mensetsu synthesizes your resume against actual company architecture, engineering whitepapers, and role specifications into an actionable technical preparation workspace.
      </p>

      <div class="flex items-center justify-center gap-3 pt-2">
        <NuxtLink to="/register">
          <AppButton size="md">
            Create Free Account
          </AppButton>
        </NuxtLink>
        <NuxtLink to="/login">
          <AppButton variant="outline" size="md">
            Sign In to Workspace
          </AppButton>
        </NuxtLink>
      </div>
    </section>


    <section class="grid grid-cols-1 md:grid-cols-3 gap-8 border-t border-slate-200 dark:border-slate-800 pt-12">
      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          01
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          Company Architecture & Culture
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          Multi-chapter deep dives synthesized from primary engineering blogs, RFCs, and infrastructure talks, backed by verifiable citations.
        </p>
      </div>

      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          02
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          Explainable Match Analysis
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          Zero arbitrary percentage dials. Clear separation between core requirements and preferred skills, with concrete resume wording improvements.
        </p>
      </div>

      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          03
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          Knowledge Graphs & Cockpit
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          Topological skill maps, prerequisite-ordered learning paths, and an interactive question bank with evaluation rubrics and scratchpad.
        </p>
      </div>
    </section>


    <section class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 sm:p-8 space-y-4 shadow-sm">
      <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
        <div class="text-xs font-mono font-semibold uppercase tracking-wider text-slate-500">
          Durable Pipeline Topology
        </div>
        <span class="text-xs font-mono text-emerald-600 dark:text-emerald-400 font-medium">TEMPORAL ORCHESTRATED</span>
      </div>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-xs font-mono">
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 1</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">SSRF Web Research</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 2</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">SurrealQL Graph Build</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 3</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">Prerequisite Syllabus</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 4</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">Targeted Question Bank</div>
        </div>
      </div>
    </section>
  </div>
</template>
