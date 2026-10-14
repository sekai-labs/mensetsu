<script setup lang="ts">
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { CreateWorkspaceRequest } from '~/features/interviews/models/interview.model'
import type { SeniorityLevel } from '~/models/common.model'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const router = useRouter()
const { createWorkspace } = useInterviewService()

const step = ref<number>(1)
const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string>('')

const companyName = ref<string>('')
const roleTitle = ref<string>('')
const seniority = ref<SeniorityLevel>('staff')
const targetDate = ref<string>('')
const prepTimeWeeks = ref<number>(2)
const urlsInput = ref<string>('')
const resumeText = ref<string>('')
const jdText = ref<string>('')
const resumeFileName = ref<string | null>(null)
const resumePdfBase64 = ref<string | null>(null)

const steps = [
  { id: 1, name: 'Target Role', desc: 'Company and level' },
  { id: 2, name: 'Evidence URLs', desc: 'Blogs and architecture' },
  { id: 3, name: 'Resume', desc: 'Candidate experience' },
  { id: 4, name: 'Job Description', desc: 'Role requirements & review' },
]

const canAdvanceStep1 = computed(() => companyName.value.trim().length > 0 && roleTitle.value.trim().length > 0)
const hasResume = computed(() => resumeText.value.trim().length > 0 || Boolean(resumePdfBase64.value))
const hasJobDescription = computed(() => jdText.value.trim().length > 0)

const handleFileUpload = async (e: Event) => {
  const target = e.target
  if (!target || !(target instanceof HTMLInputElement) || !target.files || target.files.length === 0) {
    return
  }
  const file = target.files[0]
  if (!file) return
  if (file.type !== 'application/pdf' && !file.name.toLowerCase().endsWith('.pdf')) {
    errorMessage.value = 'Please upload a PDF document.'
    return
  }
  resumeFileName.value = file.name
  const reader = new FileReader()
  reader.onload = () => {
    const result = reader.result
    if (typeof result === 'string') {
      const commaIdx = result.indexOf(',')
      resumePdfBase64.value = commaIdx >= 0 ? result.slice(commaIdx + 1) : result
    }
  }
  if (file) {
    reader.readAsDataURL(file)
  }
}

const advanceStep = () => {
  errorMessage.value = ''
  if (step.value === 1 && !canAdvanceStep1.value) {
    errorMessage.value = 'Please provide both company name and target role title.'
    return
  }
  if (step.value === 3 && !hasResume.value) {
    errorMessage.value = 'Please provide candidate resume text or upload a resume PDF before continuing.'
    return
  }
  if (step.value < 4) {
    step.value++
  }
}

const handleSubmit = async () => {
  errorMessage.value = ''
  if (!hasResume.value) {
    errorMessage.value = 'Please provide candidate resume text or upload a resume PDF.'
    return
  }
  if (!hasJobDescription.value) {
    errorMessage.value = 'Please provide target job description text to ground the workspace.'
    return
  }
  isSubmitting.value = true
  try {
    const urls = urlsInput.value
      .split('\n')
      .map((u) => u.trim())
      .filter((u) => u.length > 0)

    const targetDateIso = targetDate.value
      ? (targetDate.value.includes('T') ? targetDate.value : `${targetDate.value}T00:00:00Z`)
      : null

    const payload: CreateWorkspaceRequest = {
      company_name: companyName.value.trim(),
      role_title: roleTitle.value.trim(),
      seniority: seniority.value,
      target_date: targetDateIso,
      prep_time_weeks: prepTimeWeeks.value,
      company_urls: urls,
      resume_text: resumeText.value.trim() || null,
      resume_filename: resumeFileName.value,
      resume_pdf_base64: resumePdfBase64.value,
      job_description: jdText.value.trim() || null,
    }

    const created = await createWorkspace(payload)
    const cleanId = created.id.replace(/^workspace:/, '').replace(/`/g, '')
    await router.push(`/interviews/${cleanId}/overview`)
  } catch (err: unknown) {
    if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
      errorMessage.value = err.message
    } else {
      errorMessage.value = 'Failed to create workspace. Please check your inputs.'
    }
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="max-w-2xl mx-auto py-6 space-y-8">

    <div class="space-y-1">
      <h1 class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
        New Preparation Workspace
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400">
        Initiate an asynchronous durable research pipeline for your target company and role.
      </p>
    </div>

    <div class="grid grid-cols-4 gap-2 border-b border-slate-200 dark:border-slate-800 pb-4">
      <div
        v-for="s in steps"
        :key="s.id"
        class="space-y-1"
      >
        <div class="flex items-center gap-1.5">
          <span
            class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-mono font-bold"
            :class="step >= s.id
              ? 'bg-indigo-600 text-white'
              : 'bg-slate-200 dark:bg-slate-800 text-slate-500'"
          >
            {{ s.id }}
          </span>
          <span
            class="text-xs font-semibold truncate hidden sm:inline"
            :class="step >= s.id ? 'text-slate-900 dark:text-white' : 'text-slate-400'"
          >
            {{ s.name }}
          </span>
        </div>
      </div>
    </div>

    <div
      v-if="errorMessage"
      class="p-3 text-xs rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300"
      role="alert"
    >
      {{ errorMessage }}
    </div>

    <section v-if="step === 1" class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Step 1: Target Organization & Role
      </h2>

      <AppInput
        id="company-name"
        v-model="companyName"
        label="Target Company Name"
        placeholder="e.g. OpenAI, ScaleVector, Stripe"
        required
      />

      <AppInput
        id="role-title"
        v-model="roleTitle"
        label="Target Role Title"
        placeholder="e.g. Infrastructure Engineer, Systems Architect"
        required
      />

      <div class="space-y-1.5">
        <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
          Seniority Level
        </label>
        <div class="grid grid-cols-3 sm:grid-cols-6 gap-2">
          <button
            v-for="lvl in (['junior', 'mid', 'senior', 'staff', 'principal', 'lead'] as SeniorityLevel[])"
            :key="lvl"
            type="button"
            class="px-2 py-1.5 rounded border text-xs capitalize font-medium transition-colors"
            :class="seniority === lvl
              ? 'border-indigo-600 bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 font-bold'
              : 'border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'"
            @click="seniority = lvl"
          >
            {{ lvl }}
          </button>
        </div>
      </div>
    </section>

    <section v-else-if="step === 2" class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Step 2: Company URLs & Architecture Sources
      </h2>
      <p class="text-xs text-slate-500 leading-relaxed">
        Provide official engineering blogs, tech talks, architecture documentation, or career pages. Our research engine inspects these with SSRF-safe fetching.
      </p>

      <div class="space-y-1.5">
        <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
          Source URLs (One per line)
        </label>
        <textarea
          v-model="urlsInput"
          data-testid="urls-text"
          rows="5"
          placeholder="https://example.com/blog/architecture"
          class="block w-full rounded-md border text-xs p-3 font-mono bg-white dark:bg-slate-900 border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        ></textarea>
      </div>
    </section>

    <section v-else-if="step === 3" class="space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
          Step 3: Candidate Resume
        </h2>
        <label class="text-xs text-indigo-600 dark:text-indigo-400 hover:underline font-medium cursor-pointer">
          <span>{{ resumeFileName ? `Uploaded: ${resumeFileName}` : 'Upload PDF' }}</span>
          <input
            type="file"
            accept="application/pdf"
            class="hidden"
            @change="handleFileUpload"
          />
        </label>
      </div>

      <div class="space-y-1.5">
        <textarea
          v-model="resumeText"
          data-testid="resume-text"
          rows="7"
          placeholder="Paste candidate resume content, qualifications, and experience here..."
          class="block w-full rounded-md border text-xs p-3 font-mono bg-white dark:bg-slate-900 border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        ></textarea>
      </div>
    </section>

    <section v-else-if="step === 4" class="space-y-4">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-300">
        Step 4: Target Job Description & Review
      </h2>

      <div class="space-y-1.5">
        <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
          Job Description Text
        </label>
        <textarea
          v-model="jdText"
          data-testid="jd-text"
          rows="6"
          placeholder="Paste the target job description requirements, responsibilities, and qualifications..."
          class="block w-full rounded-md border text-xs p-3 font-mono bg-white dark:bg-slate-900 border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        ></textarea>
      </div>

      <div class="p-4 bg-slate-50 dark:bg-slate-800 rounded border border-slate-200 dark:border-slate-800 space-y-2 text-xs">
        <div class="font-semibold text-slate-900 dark:text-white">Workspace Summary Review</div>
        <div class="text-slate-600 dark:text-slate-400">
          <div><span class="font-medium">Company:</span> {{ companyName }}</div>
          <div><span class="font-medium">Target Role:</span> {{ roleTitle }} ({{ seniority }})</div>
          <div><span class="font-medium">URLs:</span> {{ urlsInput ? urlsInput.split('\n').filter(Boolean).length : 0 }} specified</div>
          <div><span class="font-medium">Resume Source:</span> {{ resumeFileName ? `PDF (${resumeFileName})` : (resumeText ? 'Text provided' : 'None') }}</div>
        </div>
      </div>
    </section>

    <div class="flex items-center justify-between pt-4 border-t border-slate-200 dark:border-slate-800">
      <button
        v-if="step > 1"
        type="button"
        class="text-xs font-medium text-slate-500 hover:text-slate-900 dark:hover:text-white"
        @click="step--"
      >
        ← Previous Step
      </button>
      <div v-else></div>

      <AppButton
        v-if="step < 4"
        type="button"
        size="sm"
        data-testid="wizard-continue"
        @click="advanceStep"
      >
        Continue →
      </AppButton>
      <AppButton
        v-else
        type="button"
        size="sm"
        data-testid="wizard-submit"
        :loading="isSubmitting"
        @click="handleSubmit"
      >
        Initialize Workspace Pipeline
      </AppButton>
    </div>
  </div>
</template>
