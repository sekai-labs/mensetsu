<script setup lang="ts">
await navigateTo('/dashboard', { replace: true })
</script>

<template>
  <main class="grid min-h-screen place-items-center" role="status" aria-live="polite">
    <p class="text-sm text-slate-500">Opening your workspace…</p>
  </main>
</template>
