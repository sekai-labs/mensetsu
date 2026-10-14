<script setup lang="ts">
import { useRoute } from 'vue-router'

definePageMeta({
  layout: 'workspace',
})

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))

await navigateTo(`/interviews/${workspaceId.value}/overview`, { replace: true })
</script>

<template>
  <div></div>
</template>
