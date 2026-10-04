<script setup lang="ts">
import { computed } from 'vue'
import { getPipelineStatusStyle } from '../utils/admin.util'

const props = withDefaults(
  defineProps<{
    status: string
    size?: 'sm' | 'md'
  }>(),
  {
    size: 'sm',
  }
)

const sizeClasses = computed(() => {
  return props.size === 'sm' ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-xs'
})

const badgeClasses = computed(() => {
  return getPipelineStatusStyle(props.status)
})
</script>

<template>
  <span
    :class="[
      'inline-flex items-center rounded border font-mono font-bold uppercase tracking-wider',
      sizeClasses,
      badgeClasses,
    ]"
  >
    <slot>{{ status }}</slot>
  </span>
</template>
