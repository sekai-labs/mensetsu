<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { SeniorityLevel } from '~/models/common.model'
import type { CreateWorkspaceRequest } from '~/features/interviews/models/interview.model'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'
import { Building2, Briefcase, FileText, Link2, UploadCloud, ChevronRight, Check } from 'lucide-vue-next'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()
const router = useRouter()
const { createWorkspace, createResumeUpload, completeResumeUpload, uploadResumeFileToSignedUrl } = useInterviewService()

const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string>('')

// Form State
const currentStep = ref<number>(1)
const companyName = ref<string>('')
const roleTitle = ref<string>('')
const seniority = ref<SeniorityLevel>('senior')
const urlsInput = ref<string>('')
const resumeMode = ref<'paste' | 'upload'>('paste')
const resumeText = ref<string>('')
const jdText = ref<string>('')
const resumeFile = ref<File | null>(null)
const resumeFileName = computed(() => resumeFile.value?.name ?? null)
const isDragging = ref<boolean>(false)
const notes = ref<string>('')

const MAX_FILE_SIZE = 5 * 1024 * 1024 // 5 MiB

const seniorityLevels: Array<{ value: SeniorityLevel; label: string; desc: string }> = [
  { value: 'junior', label: 'Junior', desc: '0-2 yrs' },
  { value: 'mid', label: 'Mid-Level', desc: '3-5 yrs' },
  { value: 'senior', label: 'Senior', desc: '5-8 yrs' },
  { value: 'staff', label: 'Staff', desc: '8-12 yrs' },
  { value: 'principal', label: 'Principal', desc: '12+ yrs' },
  { value: 'lead', label: 'Lead', desc: 'Management' },
]

const isCurrentResumeValid = computed(() => {
  if (resumeMode.value === 'paste') {
    return resumeText.value.trim().length > 0
  }
  return resumeFile.value !== null
})

const hasResume = computed(() => isCurrentResumeValid.value)

const step1Valid = computed(() => companyName.value.trim().length > 0 && roleTitle.value.trim().length > 0)
const step2Valid = computed(() => true)
const step3Valid = computed(() => isCurrentResumeValid.value)
const step4Valid = computed(() => true)

const canSubmit = computed(() => {
  return step1Valid.value && isCurrentResumeValid.value && !isSubmitting.value
})

const validateAndSetFile = (file: File) => {
  const isPdfMime = file.type === 'application/pdf'
  const hasPdfExt = file.name.toLowerCase().endsWith('.pdf')
  if (!isPdfMime && !hasPdfExt) {
    errorMessage.value = 'Please select a PDF document (.pdf).'
    return
  }
  if (file.size <= 0) {
    errorMessage.value = 'File cannot be empty.'
    return
  }
  if (file.size > MAX_FILE_SIZE) {
    errorMessage.value = 'File size exceeds maximum limit of 5 MiB.'
    return
  }

  errorMessage.value = ''
  resumeFile.value = file
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
  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return
  const file = files[0]
  if (file) {
    validateAndSetFile(file)
  }
}

const handleFileUpload = (e: Event) => {
  const target = e.target
  if (!target || !(target instanceof HTMLInputElement) || !target.files || target.files.length === 0) {
    return
  }
  const file = target.files[0]
  if (!file) return
  validateAndSetFile(file)
}

const goToNextStep = () => {
  if (currentStep.value === 1 && !step1Valid.value) {
    errorMessage.value = 'Please enter both company name and role title.'
    return
  }
  if (currentStep.value === 3 && !isCurrentResumeValid.value) {
    if (resumeMode.value === 'paste') {
      errorMessage.value = 'Please provide candidate resume text.'
    } else {
      errorMessage.value = 'Please upload a PDF document (up to 5 MiB).'
    }
    return
  }
  errorMessage.value = ''
  if (currentStep.value < 4) {
    currentStep.value += 1
  }
}

const handleSubmit = async () => {
  if (isSubmitting.value) return
  errorMessage.value = ''
  if (!step1Valid.value) {
    errorMessage.value = 'Please provide target company and role title.'
    currentStep.value = 1
    return
  }
  if (!isCurrentResumeValid.value) {
    if (resumeMode.value === 'paste') {
      errorMessage.value = 'Please provide candidate resume text.'
    } else {
      errorMessage.value = 'Please upload a resume PDF (up to 5 MiB).'
    }
    currentStep.value = 3
    return
  }

  isSubmitting.value = true
  try {
    const urls = urlsInput.value
      .split('\n')
      .map((u) => u.trim())
      .filter((u) => u.length > 0)

    let resumeUploadId: string | undefined = undefined

    if (resumeMode.value === 'upload') {
      const file = resumeFile.value
      if (!file) {
        throw new Error('Please select a resume PDF file.')
      }

      // Step 1: Request presigned upload URL from BFF
      const presignRes = await createResumeUpload({
        filename: file.name,
        size_bytes: file.size,
        content_type: 'application/pdf',
      })

      // Step 2: Binary PUT file bytes directly to opaque signed URL without auth/cookies
      await uploadResumeFileToSignedUrl(presignRes.upload_url, file)

      // Step 3: Complete upload confirmation
      const completeRes = await completeResumeUpload(presignRes.upload_id)
      if (completeRes.status !== 'completed') {
        throw new Error('Résumé upload was not confirmed by storage.')
      }
      resumeUploadId = completeRes.upload_id
    }

    const payload: CreateWorkspaceRequest = {
      company_name: companyName.value.trim(),
      role_title: roleTitle.value.trim(),
      workflow_mode: 'hybrid',
      company_urls: urls,
      seniority: seniority.value,
      resume_text: resumeMode.value === 'paste' ? resumeText.value.trim() : undefined,
      resume_upload_id: resumeMode.value === 'upload' ? resumeUploadId : undefined,
      job_description: jdText.value.trim() || undefined,
      prompt: notes.value.trim() || undefined,
    }

    const ws = await createWorkspace(payload)
    const cleanId = ws.id.replace(/^workspace:/, '').replace(/`/g, '')
    router.push(`/interviews/${cleanId}/overview`)
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { data?: { error?: { message?: string } }; message?: string }
      errorMessage.value =
        apiErr.data?.error?.message || apiErr.message || 'Failed to create workspace. Please check inputs.'
    } else {
      errorMessage.value = 'Failed to create workspace. Please check inputs.'
    }
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto py-8 px-4 sm:px-6">
    <!-- Header -->
    <div class="mb-8 border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 mb-2">
        <span class="px-2.5 py-0.5 text-xs font-mono font-semibold bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded">
          {{ $t('wizard.badge') }}
        </span>
        <span class="text-xs text-slate-500 dark:text-slate-400">{{ $t('wizard.subtitle') }}</span>
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ $t('wizard.title') }}
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl">
        {{ $t('wizard.description') }}
      </p>

      <!-- Step Navigation Indicator -->
      <div class="mt-6 flex items-center gap-2 sm:gap-4 overflow-x-auto pb-2 text-xs font-medium">
        <button
          type="button"
          @click="currentStep = 1"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors shrink-0"
          :class="[
            currentStep === 1
              ? 'bg-indigo-600 text-white font-semibold shadow-xs'
              : (step1Valid ? 'text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/40' : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800')
          ]"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-mono border"
            :class="currentStep === 1 ? 'border-white' : (step1Valid ? 'border-emerald-500' : 'border-slate-400')">
            <Check v-if="step1Valid && currentStep !== 1" class="w-3 h-3" />
            <span v-else>1</span>
          </span>
          <span>{{ $t('wizard.step1') }}</span>
        </button>

        <span class="text-slate-300 dark:text-slate-700">/</span>

        <button
          type="button"
          @click="currentStep = 2"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors shrink-0"
          :class="[
            currentStep === 2
              ? 'bg-indigo-600 text-white font-semibold shadow-xs'
              : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
          ]"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-mono border border-current">
            2
          </span>
          <span>{{ $t('wizard.step2') }}</span>
        </button>

        <span class="text-slate-300 dark:text-slate-700">/</span>

        <button
          type="button"
          @click="currentStep = 3"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors shrink-0"
          :class="[
            currentStep === 3
              ? 'bg-indigo-600 text-white font-semibold shadow-xs'
              : (step3Valid ? 'text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/40' : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800')
          ]"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-mono border"
            :class="currentStep === 3 ? 'border-white' : (step3Valid ? 'border-emerald-500' : 'border-slate-400')">
            <Check v-if="step3Valid && currentStep !== 3" class="w-3 h-3" />
            <span v-else>3</span>
          </span>
          <span>{{ $t('wizard.step3') }}</span>
        </button>

        <span class="text-slate-300 dark:text-slate-700">/</span>

        <button
          type="button"
          @click="currentStep = 4"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors shrink-0"
          :class="[
            currentStep === 4
              ? 'bg-indigo-600 text-white font-semibold shadow-xs'
              : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
          ]"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-mono border border-current">
            4
          </span>
          <span>{{ $t('wizard.step4') }}</span>
        </button>
      </div>
    </div>

    <!-- Error Banner -->
    <div v-if="errorMessage" class="mb-6 p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300 text-xs flex items-start gap-2">
      <span class="font-bold">Error:</span>
      <span>{{ errorMessage }}</span>
    </div>

    <form @submit.prevent="handleSubmit" class="space-y-6">
      <!-- STEP 1: Target Role & Seniority -->
      <div v-show="currentStep === 1" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-6">
        <div>
          <h2 class="text-sm font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-2">
            <Building2 class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
            <span>{{ $t('wizard.step1') }}</span>
          </h2>
          <p class="text-xs text-slate-500 mt-1">
            {{ $t('wizard.subtitle') }}
          </p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label for="company-name" class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
              {{ $t('wizard.companyName') }} <span class="text-rose-500">*</span>
            </label>
            <input
              id="company-name"
              v-model="companyName"
              type="text"
              required
              :placeholder="$t('wizard.companyNamePlaceholder')"
              class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg px-3.5 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
          </div>

          <div>
            <label for="role-title" class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
              {{ $t('wizard.roleTitle') }} <span class="text-rose-500">*</span>
            </label>
            <input
              id="role-title"
              v-model="roleTitle"
              type="text"
              required
              :placeholder="$t('wizard.roleTitlePlaceholder')"
              class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg px-3.5 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-2">
            {{ $t('wizard.seniorityLevel') }}
          </label>
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-2">
            <button
              v-for="lvl in seniorityLevels"
              :key="lvl.value"
              type="button"
              @click="seniority = lvl.value"
              class="p-2.5 rounded-lg border text-left transition-all"
              :class="[
                seniority === lvl.value
                  ? 'border-indigo-600 bg-indigo-50/70 dark:bg-indigo-950/60 ring-1 ring-indigo-600'
                  : 'border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-850 hover:border-slate-300 dark:hover:border-slate-700'
              ]"
            >
              <div class="text-xs font-semibold" :class="seniority === lvl.value ? 'text-indigo-700 dark:text-indigo-300' : 'text-slate-900 dark:text-white'">
                {{ lvl.label }}
              </div>
              <div class="text-[10px] text-slate-500 dark:text-slate-400">
                {{ lvl.desc }}
              </div>
            </button>
          </div>
        </div>

        <div class="flex justify-end pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            type="button"
            data-testid="wizard-continue"
            @click="goToNextStep"
            :disabled="!step1Valid"
            class="px-4 py-2 rounded-lg bg-indigo-600 text-white font-medium text-xs hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center gap-1.5 shadow-sm"
          >
            <span>{{ $t('wizard.continueSources') }}</span>
            <ChevronRight class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- STEP 2: Sources & Focus URLs -->
      <div v-show="currentStep === 2" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-6">
        <div>
          <h2 class="text-sm font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-2">
            <Link2 class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
            <span>{{ $t('wizard.sourcesTitle') }}</span>
          </h2>
          <p class="text-xs text-slate-500 mt-1">
            {{ $t('wizard.sourcesDesc') }}
          </p>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
            {{ $t('wizard.urlsLabel') }}
          </label>
          <textarea
            data-testid="urls-text"
            v-model="urlsInput"
            rows="3"
            placeholder="https://blog.cloudflare.com/&#10;https://stripe.com/blog/engineering"
            class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg p-3 text-xs font-mono text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          ></textarea>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
            {{ $t('wizard.notesLabel') }}
          </label>
          <textarea
            v-model="notes"
            rows="2"
            :placeholder="$t('wizard.notesPlaceholder')"
            class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg p-3 text-xs text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          ></textarea>
        </div>

        <div class="flex items-center justify-between pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            type="button"
            @click="currentStep = 1"
            class="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white transition"
          >
            ← {{ $t('common.back') }}
          </button>
          <button
            type="button"
            data-testid="wizard-continue"
            @click="goToNextStep"
            class="px-4 py-2 rounded-lg bg-indigo-600 text-white font-medium text-xs hover:bg-indigo-500 transition flex items-center gap-1.5 shadow-sm"
          >
            <span>{{ $t('wizard.continueResume') }}</span>
            <ChevronRight class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- STEP 3: Candidate Resume -->
      <div v-show="currentStep === 3" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-6">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div>
            <h2 class="text-sm font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-2">
              <FileText class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
              <span>{{ $t('wizard.resumeTitle') }}</span>
              <span class="text-rose-500 text-xs">*</span>
            </h2>
            <p class="text-xs text-slate-500 mt-1">
              {{ $t('wizard.resumeDesc') }}
            </p>
          </div>

          <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-800 p-0.5 rounded-lg text-xs">
            <button
              type="button"
              @click="resumeMode = 'paste'"
              class="px-2.5 py-1 rounded-md transition-colors"
              :class="resumeMode === 'paste' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white font-medium shadow-2xs' : 'text-slate-600 dark:text-slate-400'"
            >
              {{ $t('wizard.plainText') }}
            </button>
            <button
              type="button"
              @click="resumeMode = 'upload'"
              class="px-2.5 py-1 rounded-md transition-colors"
              :class="resumeMode === 'upload' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white font-medium shadow-2xs' : 'text-slate-600 dark:text-slate-400'"
            >
              {{ $t('wizard.pdfUpload') }}
            </button>
          </div>
        </div>

        <!-- Mode: Paste Text -->
        <div v-if="resumeMode === 'paste'" class="space-y-2">
          <textarea
            data-testid="resume-text"
            v-model="resumeText"
            rows="8"
            :placeholder="$t('wizard.resumePlaceholder')"
            class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg p-3 text-xs font-mono text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          ></textarea>
        </div>

        <!-- Mode: Upload PDF -->
        <div v-else class="space-y-2">
          <div
            class="border-2 border-dashed transition rounded-lg p-8 flex flex-col items-center justify-center text-center cursor-pointer"
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
              type="file"
              accept=".pdf,application/pdf"
              @change="handleFileUpload"
              class="hidden"
              id="resume-upload"
              data-testid="resume-file-input"
            />
            <label for="resume-upload" class="cursor-pointer w-full h-full flex flex-col items-center justify-center">
              <UploadCloud
                class="w-10 h-10 mb-2 transition-colors"
                :class="isDragging ? 'text-indigo-600 dark:text-indigo-400 scale-110' : 'text-slate-400'"
              />
              <span v-if="resumeFileName" class="text-xs font-mono font-medium text-indigo-600 dark:text-indigo-400">
                {{ resumeFileName }}
              </span>
              <span v-else class="text-xs font-medium text-slate-600 dark:text-slate-300">
                {{ $t('wizard.dropzone') }}
              </span>
              <span class="text-[11px] text-slate-400 mt-1">{{ $t('wizard.dropzoneLimit') }}</span>
            </label>
          </div>
        </div>

        <div class="flex items-center justify-between pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            type="button"
            @click="currentStep = 2"
            class="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white transition"
          >
            ← {{ $t('common.back') }}
          </button>
          <button
            type="button"
            data-testid="wizard-continue"
            @click="goToNextStep"
            :disabled="!step3Valid"
            class="px-4 py-2 rounded-lg bg-indigo-600 text-white font-medium text-xs hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center gap-1.5 shadow-sm"
          >
            <span>{{ $t('wizard.continueJd') }}</span>
            <ChevronRight class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- STEP 4: Job Description & Launch -->
      <div v-show="currentStep === 4" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-6">
        <div>
          <h2 class="text-sm font-bold uppercase tracking-wider text-slate-900 dark:text-white flex items-center gap-2">
            <Briefcase class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
            <span>{{ $t('wizard.jdTitle') }}</span>
          </h2>
          <p class="text-xs text-slate-500 mt-1">
            {{ $t('wizard.jdDesc') }}
          </p>
        </div>

        <div>
          <textarea
            data-testid="jd-text"
            v-model="jdText"
            rows="8"
            :placeholder="$t('wizard.jdPlaceholder')"
            class="w-full bg-slate-50 dark:bg-slate-850 border border-slate-300 dark:border-slate-700 rounded-lg p-3 text-xs font-mono text-slate-900 dark:text-white placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          ></textarea>
        </div>

        <!-- Summary overview before submission -->
        <div class="p-3.5 rounded-lg bg-slate-50 dark:bg-slate-850/60 border border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs">
          <div class="space-y-0.5">
            <div class="font-semibold text-slate-900 dark:text-white">
              {{ companyName || 'Company' }} — {{ roleTitle || 'Role' }}
            </div>
            <div class="text-[11px] text-slate-500">
              Seniority: <span class="capitalize">{{ seniority }}</span> • Resume: <span :class="hasResume ? 'text-emerald-600 font-medium' : 'text-rose-500'">{{ hasResume ? 'Provided' : 'Missing' }}</span>
            </div>
          </div>
          <button
            type="button"
            @click="currentStep = 1"
            class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline"
          >
            {{ $t('common.edit') }}
          </button>
        </div>

        <div class="flex items-center justify-between pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            type="button"
            @click="currentStep = 3"
            class="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white transition"
          >
            ← {{ $t('common.back') }}
          </button>
          <button
            type="submit"
            data-testid="wizard-submit"
            :disabled="!canSubmit"
            class="px-6 py-2.5 rounded-lg bg-indigo-600 text-white font-semibold text-xs hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center gap-2 shadow-sm"
          >
            <span v-if="isSubmitting" class="animate-spin w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full"></span>
            <span>{{ isSubmitting ? $t('wizard.submitting') : $t('wizard.submit') }}</span>
          </button>
        </div>
      </div>
    </form>
  </div>
</template>
