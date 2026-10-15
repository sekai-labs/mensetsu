<script setup lang="ts">
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { SeniorityLevel } from '~/models/common.model'
import AppInput from '~/components/ui/AppInput.vue'

definePageMeta({
  layout: 'default',
})

const router = useRouter()
const { createWorkspace } = useInterviewService()

const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string>('')

// Prompt-First State
const prompt = ref<string>('')
const companyName = ref<string>('')
const roleTitle = ref<string>('')
const seniority = ref<SeniorityLevel>('staff')
const urlsInput = ref<string>('')
const resumeText = ref<string>('')
const jdText = ref<string>('')
const resumeFileName = ref<string | null>(null)
const resumePdfBase64 = ref<string | null>(null)
const showAdvanced = ref<boolean>(false)

const hasResume = computed(() => resumeText.value.trim().length > 0 || Boolean(resumePdfBase64.value))
const canSubmit = computed(() => {
  const hasPromptOrExplicit = prompt.value.trim().length > 0 || (companyName.value.trim().length > 0 && roleTitle.value.trim().length > 0)
  return hasPromptOrExplicit && hasResume.value && !isSubmitting.value
})

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

const handleSubmit = async () => {
  errorMessage.value = ''
  if (!prompt.value.trim() && (!companyName.value.trim() || !roleTitle.value.trim())) {
    errorMessage.value = 'Please describe your interview goal or enter target company & role.'
    return
  }
  if (!hasResume.value) {
    errorMessage.value = 'Please provide candidate resume text or upload a resume PDF.'
    return
  }

  isSubmitting.value = true
  try {
    const urls = urlsInput.value
      .split('\n')
      .map((u) => u.trim())
      .filter((u) => u.length > 0)

    const payload = {
      prompt: prompt.value.trim() || undefined,
      company_name: companyName.value.trim() || undefined,
      role_title: roleTitle.value.trim() || undefined,
      workflow_mode: 'hybrid',
      company_urls: urls,
      seniority: seniority.value,
      resume_text: resumeText.value.trim() || undefined,
      resume_pdf_base64: resumePdfBase64.value || undefined,
      resume_filename: resumeFileName.value || undefined,
      job_description: jdText.value.trim() || undefined,
    }

    const ws = await createWorkspace(payload as any)
    const cleanId = ws.id.replace(/^workspace:/, '').replace(/`/g, '')
    router.push(`/interviews/${cleanId}/overview`)
  } catch (err: any) {
    errorMessage.value = err?.data?.error?.message || err?.message || 'Failed to create workspace. Please check inputs.'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto py-8 px-4 sm:px-6">
    <!-- Header -->
    <div class="mb-8 border-b border-surface-border pb-5">
      <div class="flex items-center gap-2 mb-2">
        <span class="px-2 py-0.5 text-xs font-mono font-medium bg-primary/10 text-primary border border-primary/20 rounded">Hybrid AI Engine</span>
        <span class="text-xs text-text-muted">Autonomous Planner + Deterministic Synthesis</span>
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-text-primary">Prepare for an Interview</h1>
      <p class="text-sm text-text-muted mt-1">
        Describe your target role and goals. Mensetsu routes, researches, and compiles your personalized preparation workspace.
      </p>
    </div>

    <!-- Error Banner -->
    <div v-if="errorMessage" class="mb-6 p-4 rounded-lg bg-danger/10 border border-danger/20 text-danger text-sm flex items-start gap-3">
      <span class="font-bold">Error:</span>
      <span>{{ errorMessage }}</span>
    </div>

    <form @submit.prevent="handleSubmit" class="space-y-6">
      <!-- Natural Language Goal Prompt -->
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm">
        <div class="flex justify-between items-center mb-2">
          <label class="block text-sm font-semibold text-text-primary">
            Preparation Goal & Focus Areas
          </label>
          <span class="text-xs text-primary font-medium">Natural Language Prompt</span>
        </div>
        <textarea
          v-model="prompt"
          rows="4"
          placeholder="e.g. Prepare me for Senior Staff Infrastructure Engineer at Stripe focusing on their MongoDB DocDB architecture and custom Envoy replacement proxy. Deep dive into card ledger settlement."
          class="w-full bg-surface-base border border-surface-border rounded-lg p-3 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary transition"
        ></textarea>
        <p class="text-xs text-text-muted mt-1.5">
          The Workflow Router mini-agent inspects your prompt and tasks autonomous research agents to investigate exact technical topics.
        </p>
      </div>

      <!-- Resume Dropzone & Text -->
      <div class="bg-surface-elevated/70 border border-surface-border rounded-xl p-5 shadow-sm">
        <div class="flex justify-between items-center mb-3">
          <label class="block text-sm font-semibold text-text-primary">
            Candidate Resume <span class="text-danger">*</span>
          </label>
          <span class="text-xs text-text-muted">PDF or Raw Text</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- PDF Upload -->
          <div class="border-2 border-dashed border-surface-border hover:border-primary/50 transition rounded-lg p-5 flex flex-col items-center justify-center text-center cursor-pointer bg-surface-base/50">
            <input
              type="file"
              accept=".pdf,application/pdf"
              @change="handleFileUpload"
              class="hidden"
              id="resume-upload"
            />
            <label for="resume-upload" class="cursor-pointer w-full h-full flex flex-col items-center justify-center">
              <svg class="w-8 h-8 text-text-muted mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              <span v-if="resumeFileName" class="text-xs font-mono font-medium text-primary">{{ resumeFileName }}</span>
              <span v-else class="text-xs font-medium text-text-muted">Upload PDF Resume</span>
            </label>
          </div>

          <!-- Raw Resume Text -->
          <div>
            <textarea
              v-model="resumeText"
              rows="4"
              placeholder="Or paste resume experience and skills here..."
              class="w-full h-full bg-surface-base border border-surface-border rounded-lg p-3 text-xs font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary transition"
            ></textarea>
          </div>
        </div>
      </div>

      <!-- Advanced / Optional Fields Toggle -->
      <div class="pt-1">
        <button
          type="button"
          @click="showAdvanced = !showAdvanced"
          class="text-xs font-medium text-text-muted hover:text-text-primary flex items-center gap-1.5 transition"
        >
          <span>{{ showAdvanced ? 'Hide' : 'Show' }} Advanced Settings (Company, Role, Job Description, URLs)</span>
          <svg class="w-3.5 h-3.5 transition-transform" :class="{ 'rotate-180': showAdvanced }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </button>
      </div>

      <!-- Advanced Fields Box -->
      <div v-show="showAdvanced" class="bg-surface-elevated/40 border border-surface-border rounded-xl p-5 space-y-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Company Name (Optional override)</label>
            <input
              v-model="companyName"
              type="text"
              placeholder="e.g. Stripe"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Role Title (Optional override)</label>
            <input
              v-model="roleTitle"
              type="text"
              placeholder="e.g. Staff Infrastructure Engineer"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Target Seniority Level</label>
            <select
              v-model="seniority"
              class="w-full bg-surface-base border border-surface-border rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            >
              <option value="junior">Junior</option>
              <option value="mid">Mid-Level</option>
              <option value="senior">Senior</option>
              <option value="staff">Staff</option>
              <option value="principal">Principal</option>
              <option value="lead">Lead / Management</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-medium text-text-muted mb-1">Evidence URLs (One per line)</label>
            <textarea
              v-model="urlsInput"
              rows="2"
              placeholder="https://stripe.com/blog/..."
              class="w-full bg-surface-base border border-surface-border rounded-lg p-2 text-xs font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-primary"
            ></textarea>
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-text-muted mb-1">Target Job Description (Optional)</label>
          <textarea
            v-model="jdText"
            rows="3"
            placeholder="Paste job posting text..."
            class="w-full bg-surface-base border border-surface-border rounded-lg p-2.5 text-xs font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-primary"
          ></textarea>
        </div>
      </div>

      <!-- Submit Action -->
      <div class="flex items-center justify-between pt-4 border-t border-surface-border">
        <NuxtLink to="/interviews" class="text-sm text-text-muted hover:text-text-primary transition">
          Cancel
        </NuxtLink>
        <button
          type="submit"
          :disabled="!canSubmit"
          class="px-6 py-2.5 rounded-lg bg-primary text-white font-medium text-sm hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center gap-2 shadow-sm"
        >
          <span v-if="isSubmitting" class="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"></span>
          <span>{{ isSubmitting ? 'Initializing Workspace...' : 'Launch Hybrid Preparation' }}</span>
        </button>
      </div>
    </form>
  </div>
</template>
