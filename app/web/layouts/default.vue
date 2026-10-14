<script setup lang="ts">
import AppHeader from '~/components/navigation/AppHeader.vue'
</script>

<template>
  <div class="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 font-sans selection:bg-indigo-600 selection:text-white">

    <AppHeader />


    <main class="flex-1 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-6" id="main-content">
      <slot />
    </main>


    <footer class="border-t border-slate-200 dark:border-slate-800 py-6 text-center text-xs text-slate-400 dark:text-slate-500">
      Mensetsu — Evidence-Backed Technical Interview Preparation Workspace
    </footer>
  </div>
</template>
