<script setup lang="ts">
interface Props {
  modelValue?: string | number
  label?: string
  id?: string
  type?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  error?: string
  hint?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  type: 'text',
  placeholder: '',
  required: false,
  disabled: false,
  error: '',
  hint: '',
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const inputId = computed(() => props.id || `input-${Math.random().toString(36).substring(2, 9)}`)
</script>

<template>
  <div class="w-full space-y-1.5">
    <div v-if="label" class="flex items-center justify-between">
      <label :for="inputId" class="block text-xs font-medium text-slate-700 dark:text-slate-300">
        {{ label }}
        <span v-if="required" class="text-rose-500 ml-0.5">*</span>
      </label>
      <slot name="label-right" />
    </div>

    <div class="relative">
      <input
        :id="inputId"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :required="required"
        :disabled="disabled"
        class="block w-full rounded-md border text-xs px-3 py-2 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 transition-colors focus:outline-none focus:ring-1 disabled:bg-slate-50 dark:disabled:bg-slate-800 disabled:opacity-60"
        :class="error
          ? 'border-rose-300 dark:border-rose-700 focus:border-rose-500 focus:ring-rose-500 text-rose-900 dark:text-rose-100'
          : 'border-slate-300 dark:border-slate-700 focus:border-indigo-500 focus:ring-indigo-500'"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      />
      <div v-if="$slots.suffix" class="absolute inset-y-0 right-0 pr-2.5 flex items-center pointer-events-none">
        <slot name="suffix" />
      </div>
    </div>

    <p v-if="error" class="text-[11px] text-rose-600 dark:text-rose-400">
      {{ error }}
    </p>
    <p v-else-if="hint" class="text-[11px] text-slate-500 dark:text-slate-400">
      {{ hint }}
    </p>
  </div>
</template>
