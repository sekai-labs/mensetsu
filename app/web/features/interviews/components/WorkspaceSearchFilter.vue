<script setup lang="ts">
defineProps<{
  searchQuery: string
  statusFilter: string
}>()

const emit = defineEmits<{
  (e: 'update:searchQuery', value: string): void
  (e: 'update:statusFilter', value: string): void
}>()

const handleSearchInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:searchQuery', target.value)
}

const handleStatusChange = (event: Event) => {
  const target = event.target as HTMLSelectElement
  emit('update:statusFilter', target.value)
}
</script>

<template>
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
    <div class="flex items-center gap-2 flex-1 max-w-md">
      <div class="relative w-full">
        <input
          :value="searchQuery"
          @input="handleSearchInput"
          type="text"
          placeholder="Search by company or role..."
          class="w-full px-3 py-2 pl-9 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
        />
        <span class="absolute left-3 top-2.5 text-slate-400 text-xs pointer-events-none">
          🔍
        </span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <label for="status-filter-select" class="text-xs text-slate-500 font-medium">Status:</label>
      <select
        id="status-filter-select"
        :value="statusFilter"
        @change="handleStatusChange"
        class="px-2.5 py-1.5 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-700 dark:text-slate-300 focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
      >
        <option value="all">All Statuses</option>
        <option value="ready">Ready</option>
        <option value="processing">Processing</option>
        <option value="pending">Pending</option>
        <option value="failed">Failed</option>
      </select>
    </div>
  </div>
</template>
