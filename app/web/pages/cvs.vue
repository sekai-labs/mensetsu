<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { ResumeUploadItemDto } from '~/features/interviews/models/interview.model'
import AppButton from '~/components/ui/AppButton.vue'
import {
  FileText,
  UploadCloud,
  Download,
  Trash2,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
  X,
} from 'lucide-vue-next'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()
const {
  listResumeUploads,
  deleteResumeUpload,
  getResumeDownloadUrl,
  createResumeUpload,
  uploadResumeFileToSignedUrl,
  completeResumeUpload,
} = useInterviewService()

const MAX_FILE_SIZE = 5 * 1024 * 1024

const cvList = ref<ResumeUploadItemDto[]>([])
const isLoading = ref<boolean>(true)
const fetchError = ref<string>('')

const cvUploadInput = ref<HTMLInputElement | null>(null)

const openCvFilePicker = () => {
  if (!isProcessingUploads.value) cvUploadInput.value?.click()
}

interface FileUploadStatus {
  id: string
  name: string
  size: number
  status: 'pending' | 'uploading' | 'success' | 'error'
  error?: string
}
const uploadQueue = ref<FileUploadStatus[]>([])

const deleteTarget = ref<ResumeUploadItemDto | null>(null)
const isDeleting = ref<boolean>(false)
const deleteError = ref<string>('')

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

const formatDate = (dateStr: string): string => {
  try {
    return new Date(dateStr).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

const loadCvList = async () => {
  isLoading.value = true
  fetchError.value = ''
  try {
    cvList.value = await listResumeUploads()
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { statusCode?: number; data?: { error?: { message?: string } }; message?: string }
      if (apiErr.statusCode === 401) {
        fetchError.value = t('cvs.unauthorizedError') || 'Session expired. Please sign in.'
      } else {
        fetchError.value = apiErr.data?.error?.message || apiErr.message || t('cvs.fetchError')
      }
    } else {
      fetchError.value = t('cvs.fetchError')
    }
  } finally {
    isLoading.value = false
  }
}

const validatePdfFile = (file: File): string | null => {
  if (!file.name.toLowerCase().endsWith('.pdf')) {
    return t('cvs.invalidExtension')
  }
  if (file.type !== 'application/pdf') {
    return t('cvs.invalidMimeType')
  }
  if (file.size <= 0) {
    return t('cvs.fileEmpty')
  }
  if (file.size > MAX_FILE_SIZE) {
    return t('cvs.fileTooLarge')
  }
  return null
}

const processFiles = async (files: FileList | File[]) => {
  if (files.length === 0 || isProcessingUploads.value) return

  const fileArray = Array.from(files)
  const newStatuses: FileUploadStatus[] = fileArray.map((f, idx) => ({
    id: `${Date.now()}-${idx}-${f.name}`,
    name: f.name,
    size: f.size,
    status: 'pending',
  }))
  uploadQueue.value = [...uploadQueue.value, ...newStatuses]

  isProcessingUploads.value = true

  for (let i = 0; i < fileArray.length; i++) {
    const file = fileArray[i]
    const statusItem = newStatuses[i]
    statusItem.status = 'uploading'

    const validationError = validatePdfFile(file)
    if (validationError) {
      statusItem.status = 'error'
      statusItem.error = validationError
      continue
    }

    try {
      const presign = await createResumeUpload({
        filename: file.name,
        size_bytes: file.size,
        content_type: 'application/pdf',
      })

      await uploadResumeFileToSignedUrl(presign.upload_url, file)

      const completeRes = await completeResumeUpload(presign.upload_id)
      if (completeRes.status !== 'completed') {
        throw new Error(t('cvs.completionFailed') || 'Upload could not be confirmed')
      }

      statusItem.status = 'success'
    } catch (err: unknown) {
      statusItem.status = 'error'
      if (err && typeof err === 'object') {
        const apiErr = err as { data?: { error?: { message?: string } }; message?: string }
        statusItem.error = apiErr.data?.error?.message || apiErr.message || t('cvs.uploadFailed')
      } else {
        statusItem.error = t('cvs.uploadFailed')
      }
    }
  }

  isProcessingUploads.value = false
  await loadCvList()
}

const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target?.files) {
    processFiles(target.files)
    target.value = ''
  }
}

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = true
}

const handleDragLeave = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
}

const handleDrop = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
  if (e.dataTransfer?.files) {
    processFiles(e.dataTransfer.files)
  }
}

const clearUploadQueue = () => {
  uploadQueue.value = []
}

const openDeleteConfirm = (cv: ResumeUploadItemDto) => {
  deleteTarget.value = cv
  deleteError.value = ''
}

const closeDeleteConfirm = () => {
  deleteTarget.value = null
  deleteError.value = ''
}

const executeDelete = async () => {
  if (!deleteTarget.value || isDeleting.value) return
  isDeleting.value = true
  deleteError.value = ''
  try {
    await deleteResumeUpload(deleteTarget.value.upload_id)
    closeDeleteConfirm()
    await loadCvList()
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { statusCode?: number; data?: { error?: { message?: string } }; message?: string }
      if (apiErr.statusCode === 409) {
        deleteError.value = t('cvs.deleteConflict')
      } else {
        deleteError.value = apiErr.data?.error?.message || apiErr.message || t('cvs.deleteFailed')
      }
    } else {
      deleteError.value = t('cvs.deleteFailed')
    }
  } finally {
    isDeleting.value = false
  }
}

onMounted(() => {
  loadCvList()
})
</script>

<template>
  <div class="max-w-5xl mx-auto py-8 px-4 sm:px-6 space-y-8">
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 mb-2">
        <span class="px-2.5 py-0.5 text-xs font-mono font-semibold bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded">
          {{ $t('cvs.badge') }}
        </span>
      </div>
      <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ $t('cvs.title') }}
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl">
        {{ $t('cvs.description') }}
      </p>
    </div>

    <!-- Upload Zone -->
    <div
      class="border-2 border-dashed transition rounded-lg p-8 flex flex-col items-center justify-center text-center select-none"
      :class="[
        isDragging
          ? 'border-indigo-500 bg-indigo-50/50 dark:bg-indigo-950/30 ring-2 ring-indigo-400'
          : 'border-slate-300 dark:border-slate-700 hover:border-indigo-500 bg-slate-50/50 dark:bg-slate-850/50'
      ]"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
    >
      <input
        ref="cvUploadInput"
        type="file"
        multiple
        accept=".pdf,application/pdf"
        @change="handleFileSelect"
        class="hidden"
        id="cv-upload-input"
        data-testid="cv-multi-upload-input"
      />
      <AppButton
        variant="outline"
        size="md"
        :disabled="isProcessingUploads"
        aria-controls="cv-upload-input"
        aria-describedby="cv-upload-limit"
        @click="openCvFilePicker"
      >
        <UploadCloud aria-hidden="true" class="w-4 h-4" />
        {{ $t('cvs.dropzone') }}
      </AppButton>
      <span id="cv-upload-limit" class="text-[11px] text-slate-500 dark:text-slate-400 mt-2">
        {{ $t('cvs.dropzoneLimit') }}
      </span>
    </div>

    <!-- Upload Progress & Errors -->
    <div v-if="uploadQueue.length > 0" class="space-y-2">
      <div class="flex items-center justify-between text-xs font-semibold text-slate-700 dark:text-slate-300">
        <span>{{ $t('cvs.uploadStatus') }}</span>
        <button
          v-if="!isProcessingUploads"
          type="button"
          @click="clearUploadQueue"
          class="text-[11px] text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 underline"
        >
          {{ $t('common.clear') }}
        </button>
      </div>

      <div class="border border-slate-200 dark:border-slate-800 rounded-lg divide-y divide-slate-100 dark:divide-slate-800 bg-white dark:bg-slate-900">
        <div
          v-for="item in uploadQueue"
          :key="item.id"
          class="p-3 flex items-center justify-between text-xs"
        >
          <div class="flex items-center gap-2.5 min-w-0 flex-1">
            <FileText class="w-4 h-4 text-slate-400 shrink-0" />
            <div class="truncate">
              <span class="font-medium text-slate-900 dark:text-white truncate block">
                {{ item.name }}
              </span>
              <span class="text-[11px] font-mono text-slate-500">
                {{ formatBytes(item.size) }}
              </span>
            </div>
          </div>

          <div class="ml-4 shrink-0">
            <div v-if="item.status === 'uploading'" class="flex items-center gap-1.5 text-indigo-600 dark:text-indigo-400">
              <div class="animate-spin rounded-full h-3.5 w-3.5 border-2 border-indigo-600 border-t-transparent" />
              <span>{{ $t('cvs.uploading') }}</span>
            </div>
            <div v-else-if="item.status === 'success'" class="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400 font-medium">
              <CheckCircle2 class="w-4 h-4" />
              <span>{{ $t('cvs.uploaded') }}</span>
            </div>
            <div v-else-if="item.status === 'error'" role="alert" class="flex items-center gap-1.5 text-rose-600 dark:text-rose-400">
              <AlertCircle class="w-4 h-4" />
              <span class="text-[11px] font-medium">{{ item.error }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- CV List Section -->
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-2">
          <FileText class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          <span>{{ $t('cvs.libraryTitle') }}</span>
          <span v-if="!isLoading" class="text-xs font-mono font-normal text-slate-400">
            ({{ cvList.length }})
          </span>
        </h2>
        <button
          type="button"
          @click="loadCvList"
          :disabled="isLoading"
          class="p-1.5 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition"
          :aria-label="$t('common.refresh')"
        >
          <RefreshCw class="w-4 h-4" :class="isLoading ? 'animate-spin' : ''" />
        </button>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="p-12 border border-slate-200 dark:border-slate-800 rounded-lg flex flex-col items-center justify-center text-slate-500 gap-2 bg-white dark:bg-slate-900">
        <div class="animate-spin rounded-full h-5 w-5 border-2 border-indigo-600 border-t-transparent" />
        <span class="text-xs">{{ $t('cvs.loading') }}</span>
      </div>

      <!-- Error State -->
      <div
        v-else-if="fetchError"
        role="alert"
        class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between"
      >
        <span>{{ fetchError }}</span>
        <button
          type="button"
          @click="loadCvList"
          class="ml-3 px-2.5 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium transition"
        >
          {{ $t('common.retry') }}
        </button>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="cvList.length === 0"
        class="p-12 border-2 border-dashed border-slate-300 dark:border-slate-700 rounded-lg text-center space-y-2"
      >
        <FileText class="w-10 h-10 mx-auto text-slate-400" />
        <p class="text-xs font-medium text-slate-700 dark:text-slate-300">
          {{ $t('cvs.noCvs') }}
        </p>
        <p class="text-[11px] text-slate-400 max-w-sm mx-auto">
          {{ $t('cvs.noCvsHint') }}
        </p>
      </div>

      <!-- CV Items Table -->
      <div
        v-else
        class="border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden bg-white dark:bg-slate-900 shadow-xs"
      >
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="border-b border-slate-200 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-850/60 text-[11px] font-mono uppercase tracking-wider text-slate-500">
              <th scope="col" class="py-2.5 px-4 font-semibold">{{ $t('cvs.tableFilename') }}</th>
              <th scope="col" class="py-2.5 px-4 font-semibold">{{ $t('cvs.tableSize') }}</th>
              <th scope="col" class="py-2.5 px-4 font-semibold">{{ $t('cvs.tableUploaded') }}</th>
              <th scope="col" class="py-2.5 px-4 font-semibold text-right">{{ $t('cvs.tableActions') }}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100 dark:divide-slate-800 text-xs">
            <tr
              v-for="cv in cvList"
              :key="cv.upload_id"
              class="hover:bg-slate-50/60 dark:hover:bg-slate-850/40 transition-colors"
            >
              <td class="py-3 px-4">
                <div class="flex items-center gap-2.5 min-w-0">
                  <FileText class="w-4 h-4 text-indigo-600 dark:text-indigo-400 shrink-0" />
                  <span class="font-medium text-slate-900 dark:text-white truncate">
                    {{ cv.filename }}
                  </span>
                </div>
              </td>
              <td class="py-3 px-4 font-mono text-slate-500 text-[11px]">
                {{ formatBytes(cv.size_bytes) }}
              </td>
              <td class="py-3 px-4 text-slate-500 text-[11px]">
                {{ formatDate(cv.created_at) }}
              </td>
              <td class="py-3 px-4 text-right">
                <div class="inline-flex items-center gap-1">
                  <a
                    :href="getResumeDownloadUrl(cv.upload_id)"
                    download
                    class="p-1.5 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    :aria-label="`${$t('cvs.download')} ${cv.filename}`"
                  >
                    <Download class="w-4 h-4" />
                  </a>
                  <button
                    type="button"
                    @click="openDeleteConfirm(cv)"
                    class="p-1.5 rounded text-slate-600 dark:text-slate-300 hover:text-rose-600 dark:hover:text-rose-400 transition focus:outline-none focus:ring-2 focus:ring-rose-500"
                    :aria-label="`${$t('cvs.delete')} ${cv.filename}`"
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div
      v-if="deleteTarget"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/50 backdrop-blur-xs animate-in fade-in duration-100"
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-dialog-title"
    >
      <div class="bg-white dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800 max-w-md w-full p-6 shadow-xl space-y-4">
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-2 text-rose-600 dark:text-rose-400">
            <AlertCircle class="w-5 h-5 shrink-0" />
            <h3 id="delete-dialog-title" class="text-sm font-bold text-slate-900 dark:text-white">
              {{ $t('cvs.deleteModalTitle') }}
            </h3>
          </div>
          <button
            type="button"
            @click="closeDeleteConfirm"
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-1"
            :aria-label="$t('common.cancel')"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <p class="text-xs text-slate-600 dark:text-slate-400">
          {{ $t('cvs.deleteModalDesc', { filename: deleteTarget.filename }) }}
        </p>

        <!-- 409 Conflict or Other Delete Error -->
        <div
          v-if="deleteError"
          role="alert"
          class="p-3 rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs"
        >
          {{ deleteError }}
        </div>

        <div class="flex items-center justify-end gap-2 pt-2">
          <AppButton
            variant="outline"
            size="sm"
            @click="closeDeleteConfirm"
            :disabled="isDeleting"
          >
            {{ $t('common.cancel') }}
          </AppButton>
          <AppButton
            variant="danger"
            size="sm"
            @click="executeDelete"
            :loading="isDeleting"
          >
            {{ $t('common.delete') }}
          </AppButton>
        </div>
      </div>
    </div>
  </div>
</template>