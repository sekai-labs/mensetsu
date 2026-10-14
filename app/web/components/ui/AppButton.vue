<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger'
  size?: 'xs' | 'sm' | 'md' | 'lg'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'sm',
  type: 'button',
  disabled: false,
  loading: false,
})

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-700 text-white shadow-sm disabled:bg-indigo-400'
    case 'secondary':
      return 'bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-800 dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-200'
    case 'outline':
      return 'border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300'
    case 'ghost':
      return 'hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
    case 'danger':
      return 'bg-rose-600 hover:bg-rose-500 active:bg-rose-700 text-white shadow-sm disabled:bg-rose-400'
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'xs':
      return 'px-2 py-1 text-xs gap-1'
    case 'sm':
      return 'px-3 py-1.5 text-xs font-medium gap-1.5'
    case 'md':
      return 'px-4 py-2 text-sm font-medium gap-2'
    case 'lg':
      return 'px-5 py-2.5 text-base font-semibold gap-2.5'
  }
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    class="inline-flex items-center justify-center rounded-md transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 focus-visible:ring-offset-1 select-none disabled:opacity-60 disabled:cursor-not-allowed"
    :class="[variantClasses, sizeClasses]"
  >
    <svg
      v-if="loading"
      class="animate-spin -ml-0.5 w-3.5 h-3.5"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
    </svg>
    <slot />
  </button>
</template>
