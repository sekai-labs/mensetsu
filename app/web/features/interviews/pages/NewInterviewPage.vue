<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import StepperNavigation from '~/components/ui/StepperNavigation.vue'
import type { SeniorityLevel } from '../models/interview.model'

const router = useRouter()
const { createWorkspace } = useInterviewService()

const currentStep = ref(1)
const steps = [
  { id: 1, title: 'Target Role', subtitle: 'Company & Title' },
  { id: 2, title: 'Sources', subtitle: 'Engineering URLs' },
  { id: 3, title: 'Candidate Profile', subtitle: 'Resume & Experience' },
  { id: 4, title: 'Job Description', subtitle: 'Requirements & Review' },
]

const companyName = ref('')
const roleTitle = ref('')
const seniority = ref<SeniorityLevel>('senior')
const prepTimeWeeks = ref(3)
const companyUrls = ref('')
const resumeText = ref('')
const resumePdfBase64 = ref<string | null>(null)
const resumeFilename = ref<string | null>(null)
const resumeFileSize = ref<number | null>(null)
const isPdfUploading = ref(false)
const resumeUploadMode = ref<'pdf' | 'text'>('pdf')

const jdMode = ref<'text' | 'url'>('text')
const jdUrl = ref('')
const jobDescription = ref('')
const isSubmitting = ref(false)
const errorMessage = ref<string | null>(null)

const nextStep = () => {
  if (currentStep.value === 1) {
    if (!companyName.value.trim() || !roleTitle.value.trim()) {
      errorMessage.value = 'Company name and target role title are required.'
      return
    }
  }
  errorMessage.value = null
  currentStep.value = Math.min(4, currentStep.value + 1)
}

const prevStep = () => {
  errorMessage.value = null
  currentStep.value = Math.max(1, currentStep.value - 1)
}

const handleSubmit = async () => {
  isSubmitting.value = true
  errorMessage.value = null

  const urls = companyUrls.value
    .split('\n')
    .map((u) => u.trim())
    .filter((u) => u.length > 0)

  try {
    const ws = await createWorkspace({
      company_name: companyName.value.trim(),
      role_title: roleTitle.value.trim(),
      seniority: seniority.value,
      prep_time_weeks: Number(prepTimeWeeks.value),
      company_urls: urls,
      resume_text: resumeText.value.trim() || undefined,
      resume_pdf_base64: resumePdfBase64.value || undefined,
      resume_filename: resumeFilename.value || undefined,
      job_description:
        jdMode.value === 'text' ? jobDescription.value.trim() || undefined : undefined,
      jd_url: jdMode.value === 'url' ? jdUrl.value.trim() || undefined : undefined,
    })

    await router.push(`/interviews/${ws.id}/overview`)
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err) {
      errorMessage.value = String((err as { message: unknown }).message)
    } else {
      errorMessage.value = 'Failed to launch preparation workflow. Check backend status.'
    }
  } finally {
    isSubmitting.value = false
  }
}

const processFile = (file: File) => {
  errorMessage.value = null
  resumeFilename.value = file.name
  resumeFileSize.value = file.size
  isPdfUploading.value = true

  const reader = new FileReader()
  reader.onload = () => {
    const result = reader.result as string
    if (file.name.toLowerCase().endsWith('.pdf') || file.type === 'application/pdf') {
      const base64Data = result.split(',')[1] || result
      resumePdfBase64.value = base64Data
    } else {
      resumeText.value = result
    }
    isPdfUploading.value = false
  }
  reader.onerror = () => {
    errorMessage.value = 'Failed to read uploaded file.'
    isPdfUploading.value = false
  }

  if (file.name.toLowerCase().endsWith('.pdf') || file.type === 'application/pdf') {
    reader.readAsDataURL(file)
  } else {
    reader.readAsText(file)
  }
}

const onFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  processFile(file)
}

const onDrop = (event: DragEvent) => {
  event.preventDefault()
  const file = event.dataTransfer?.files?.[0]
  if (!file) return
  processFile(file)
}

const removeSelectedFile = () => {
  resumePdfBase64.value = null
  resumeFilename.value = null
  resumeFileSize.value = null
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1048576).toFixed(1) + ' MB'
}
</script>

<template>
  <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    <div>
      <NuxtLink
        to="/interviews"
        class="text-xs font-semibold text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
      >
        ← Back to Interviews
      </NuxtLink>
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white mt-2">
        New Interview Preparation Workspace
      </h1>
      <p class="text-xs text-slate-500 mt-1">
        Configure the target role and background. The research engine will gather verified citations and construct your study plan.
      </p>
    </div>

    <StepperNavigation :current-step="currentStep" :steps="steps" />

    <div
      v-if="errorMessage"
      class="p-3.5 rounded-md bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-400 text-xs"
    >
      {{ errorMessage }}
    </div>

    <form @submit.prevent="handleSubmit" class="space-y-6">
      <div
        v-if="currentStep === 1"
        class="space-y-5 p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm"
      >
        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
            Target Company Name *
          </label>
          <input
            v-model="companyName"
            type="text"
            required
            placeholder="e.g. OpenAI, Stripe, Datadog"
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
          />
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
            Role Title *
          </label>
          <input
            v-model="roleTitle"
            type="text"
            required
            placeholder="e.g. Senior Infrastructure Engineer, Tech Lead"
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
              Target Seniority Level
            </label>
            <select
              v-model="seniority"
              class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
            >
              <option value="junior">Junior (L3 / Associate)</option>
              <option value="mid">Mid-level (L4)</option>
              <option value="senior">Senior (L5)</option>
              <option value="staff">Staff (L6)</option>
              <option value="principal">Principal (L7)</option>
              <option value="lead">Engineering Manager / Lead</option>
            </select>
          </div>

          <div>
            <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
              Preparation Runway
            </label>
            <select
              v-model="prepTimeWeeks"
              class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
            >
              <option :value="1">1 Week (Intensive Sprint)</option>
              <option :value="2">2 Weeks (Accelerated Focus)</option>
              <option :value="3">3 Weeks (Standard Plan)</option>
              <option :value="4">4 Weeks (Comprehensive Review)</option>
              <option :value="6">6 Weeks (Deep Architecture Drill)</option>
            </select>
          </div>
        </div>
      </div>

      <div
        v-else-if="currentStep === 2"
        class="space-y-4 p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm"
      >
        <div>
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1">
            Target Company Engineering URLs (Optional)
          </label>
          <p class="text-[11px] text-slate-500 mb-2">
            Provide company blogs, GitHub orgs, or documentation sites to ground company research. One URL per line.
          </p>
          <textarea
            v-model="companyUrls"
            rows="5"
            placeholder="https://stripe.com/blog/engineering&#10;https://github.com/stripe"
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
          ></textarea>
        </div>
      </div>

      <div
        v-else-if="currentStep === 3"
        class="space-y-4 p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm"
      >
        <div class="flex items-center justify-between">
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
            Candidate Experience & Resume *
          </label>
          <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-800 p-0.5 rounded text-xs">
            <button
              type="button"
              @click="resumeUploadMode = 'pdf'"
              class="px-2 py-0.5 rounded font-medium transition"
              :class="resumeUploadMode === 'pdf' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-500'"
            >
              PDF Upload
            </button>
            <button
              type="button"
              @click="resumeUploadMode = 'text'"
              class="px-2 py-0.5 rounded font-medium transition"
              :class="resumeUploadMode === 'text' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-500'"
            >
              Paste Text
            </button>
          </div>
        </div>

        <div v-if="resumeUploadMode === 'pdf'" class="space-y-3">
          <div
            v-if="!resumeFilename"
            @dragover.prevent
            @drop="onDrop"
            class="border-2 border-dashed border-slate-300 dark:border-slate-700 rounded-lg p-8 text-center hover:border-indigo-400 transition cursor-pointer"
            @click="($refs.fileInput as HTMLInputElement)?.click()"
          >
            <input
              ref="fileInput"
              type="file"
              accept=".pdf,.txt"
              class="hidden"
              @change="onFileSelect"
            />
            <div class="text-xs text-slate-600 dark:text-slate-400 space-y-1">
              <div class="font-medium text-slate-900 dark:text-white">
                Click to browse or drop resume PDF here
              </div>
              <div class="text-[11px] text-slate-500">PDF or plain text up to 10 MB</div>
            </div>
          </div>

          <div
            v-else
            class="p-4 rounded-md border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/60 flex items-center justify-between"
          >
            <div class="flex items-center gap-3">
              <span class="text-indigo-600 dark:text-indigo-400 text-lg">📄</span>
              <div>
                <div class="text-xs font-semibold text-slate-900 dark:text-white">
                  {{ resumeFilename }}
                </div>
                <div class="text-[11px] text-slate-500">
                  {{ resumeFileSize ? formatBytes(resumeFileSize) : 'Ready' }}
                </div>
              </div>
            </div>

            <button
              type="button"
              @click="removeSelectedFile"
              class="text-xs text-rose-600 hover:text-rose-700 dark:text-rose-400 font-medium"
            >
              Remove
            </button>
          </div>

          <div v-if="isPdfUploading" class="text-xs text-indigo-600 dark:text-indigo-400 animate-pulse">
            Processing resume file...
          </div>
        </div>

        <div v-else>
          <textarea
            v-model="resumeText"
            rows="9"
            placeholder="Paste your resume Markdown or raw text..."
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs font-sans text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm leading-relaxed"
          ></textarea>
        </div>
      </div>

      <div
        v-else-if="currentStep === 4"
        class="space-y-4 p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm"
      >
        <div class="flex items-center justify-between">
          <label class="block text-xs font-semibold text-slate-700 dark:text-slate-300">
            Target Job Description (JD)
          </label>
          <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-800 p-0.5 rounded text-xs">
            <button
              type="button"
              @click="jdMode = 'text'"
              class="px-2 py-0.5 rounded font-medium transition"
              :class="jdMode === 'text' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-500'"
            >
              Raw Text
            </button>
            <button
              type="button"
              @click="jdMode = 'url'"
              class="px-2 py-0.5 rounded font-medium transition"
              :class="jdMode === 'url' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-500'"
            >
              URL Link
            </button>
          </div>
        </div>

        <div v-if="jdMode === 'text'">
          <textarea
            v-model="jobDescription"
            rows="9"
            placeholder="Paste the target job description or requirements section here..."
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs font-sans text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm leading-relaxed"
          ></textarea>
        </div>

        <div v-else>
          <input
            v-model="jdUrl"
            type="url"
            placeholder="https://jobs.lever.co/company/role-id"
            class="w-full px-3 py-2 rounded-md border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-xs font-mono text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
          />
          <p class="text-[11px] text-slate-500 mt-1">
            The crawler will fetch the job description directly from this listing.
          </p>
        </div>
      </div>

      <div class="flex items-center justify-between pt-2">
        <button
          v-if="currentStep > 1"
          type="button"
          @click="prevStep"
          class="px-4 py-2 text-xs font-semibold rounded-md border border-slate-300 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 transition"
        >
          ← Back
        </button>
        <div v-else></div>

        <button
          v-if="currentStep < 4"
          type="button"
          @click="nextStep"
          class="px-4 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 transition shadow-sm"
        >
          Continue →
        </button>

        <button
          v-else
          type="submit"
          :disabled="isSubmitting"
          class="px-5 py-2 text-xs font-semibold rounded-md bg-indigo-600 text-white hover:bg-indigo-500 disabled:opacity-50 transition shadow-sm flex items-center gap-2"
        >
          <span v-if="isSubmitting" class="animate-spin text-sm">↻</span>
          <span>Launch Autonomous Preparation →</span>
        </button>
      </div>
    </form>
  </div>
</template>
