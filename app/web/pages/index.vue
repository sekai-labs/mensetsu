<script setup lang="ts">
import AppButton from '~/components/ui/AppButton.vue'

definePageMeta({
  layout: 'default',
})
</script>

<template>
  <div class="py-12 sm:py-20 max-w-5xl mx-auto space-y-20">
    <!-- Hero Section -->
    <section class="text-center space-y-6 max-w-3xl mx-auto">
      <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-indigo-200 dark:border-indigo-900 bg-indigo-50/60 dark:bg-indigo-950/40 text-xs font-mono font-medium text-indigo-700 dark:text-indigo-300">
        <span class="w-1.5 h-1.5 rounded-full bg-indigo-600 dark:bg-indigo-400"></span>
        <span>{{ $t('landing.badge') }}</span>
      </div>

      <h1 class="text-4xl sm:text-5xl font-bold tracking-tight text-slate-900 dark:text-white leading-[1.15]">
        {{ $t('landing.headline') }}
      </h1>

      <p class="text-base sm:text-lg text-slate-600 dark:text-slate-400 leading-relaxed max-w-2xl mx-auto">
        {{ $t('landing.subtitle') }}
      </p>

      <div class="flex items-center justify-center gap-3 pt-2">
        <NuxtLink to="/register">
          <AppButton size="md">
            {{ $t('landing.createAccount') }}
          </AppButton>
        </NuxtLink>
        <NuxtLink to="/login">
          <AppButton variant="outline" size="md">
            {{ $t('landing.signIn') }}
          </AppButton>
        </NuxtLink>
      </div>
    </section>

    <!-- 3 Core Architectural Pillars -->
    <section class="grid grid-cols-1 md:grid-cols-3 gap-8 border-t border-slate-200 dark:border-slate-800 pt-12">
      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          01
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          {{ $t('landing.features.f1Title') }}
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          {{ $t('landing.features.f1Desc') }}
        </p>
      </div>

      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          02
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          {{ $t('landing.features.f2Title') }}
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          {{ $t('landing.features.f2Desc') }}
        </p>
      </div>

      <div class="space-y-3">
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-400 font-bold uppercase tracking-wider">
          03
        </div>
        <h2 class="text-lg font-semibold text-slate-900 dark:text-white">
          {{ $t('landing.features.f3Title') }}
        </h2>
        <p class="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
          {{ $t('landing.features.f3Desc') }}
        </p>
      </div>
    </section>

    <!-- Pipeline Architecture -->
    <section class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 sm:p-8 space-y-4 shadow-sm">
      <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
        <div class="text-xs font-mono font-semibold uppercase tracking-wider text-slate-500">
          {{ $t('landing.pipeline.title') }}
        </div>
        <span class="text-xs font-mono text-emerald-600 dark:text-emerald-400 font-medium">
          {{ $t('landing.pipeline.badge') }}
        </span>
      </div>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-xs font-mono">
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 1</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">{{ $t('landing.pipeline.stage1') }}</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 2</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">{{ $t('landing.pipeline.stage2') }}</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 3</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">{{ $t('landing.pipeline.stage3') }}</div>
        </div>
        <div class="p-3 bg-slate-50 dark:bg-slate-850 rounded border border-slate-200 dark:border-slate-700/60">
          <div class="text-slate-400 mb-1">STAGE 4</div>
          <div class="font-semibold text-slate-800 dark:text-slate-200">{{ $t('landing.pipeline.stage4') }}</div>
        </div>
      </div>
    </section>
  </div>
</template>
