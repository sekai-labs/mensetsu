<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useInterviewService } from '~/features/interviews/services/interview.service'
import type { UpdateCandidateProfileRequestDto } from '~/features/interviews/models/interview.model'
import AppButton from '~/components/ui/AppButton.vue'
import AppInput from '~/components/ui/AppInput.vue'
import {
  User,
  Mail,
  Phone,
  MapPin,
  Linkedin,
  Github,
  Globe,
  Briefcase,
  Tags,
  CheckCircle2,
  AlertCircle,
  Plus,
  X,
  Lock,
} from 'lucide-vue-next'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()
const { getUserProfile, getCandidateProfile, updateCandidateProfile } = useInterviewService()

const isLoading = ref<boolean>(true)
const isSaving = ref<boolean>(false)
const loadError = ref<string>('')
const saveSuccess = ref<boolean>(false)
const saveError = ref<string>('')

// Read-only Identity Fields
const identityName = ref<string>('')
const identityEmail = ref<string>('')
const identityRole = ref<string>('')

// Editable Contact Fields
const phone = ref<string>('')
const location = ref<string>('')
const linkedinUrl = ref<string>('')
const githubUrl = ref<string>('')
const portfolioUrl = ref<string>('')

// Editable Job Targets
const targetRole = ref<string>('')
const targetLocations = ref<string[]>([])
const requiredKeywords = ref<string[]>([])
const excludedKeywords = ref<string[]>([])

// Inputs for tag addition
const newLocationInput = ref<string>('')
const newRequiredInput = ref<string>('')
const newExcludedInput = ref<string>('')

// Validation regex & helpers
const phoneRegex = /^[\d\s+\-.()]{7,20}$/
const validatePhone = (val: string): boolean => {
  if (!val.trim()) return true
  const trimmed = val.trim()
  if (!phoneRegex.test(trimmed)) return false
  const digitCount = (trimmed.match(/\d/g) || []).length
  return digitCount >= 7
}

const validateHttpsUrl = (val: string): boolean => {
  if (!val.trim()) return true
  const trimmed = val.trim()
  if (trimmed.length > 2048) return false
  try {
    const parsed = new URL(trimmed)
    return parsed.protocol === 'https:' && !parsed.username && !parsed.password
  } catch {
    return false
  }
}

const phoneError = computed(() => {
  if (!phone.value.trim()) return ''
  return validatePhone(phone.value) ? '' : t('profile.errors.invalidPhone')
})

const locationError = computed(() => {
  return location.value.trim().length <= 120 ? '' : t('profile.errors.locationTooLong')
})

const linkedinError = computed(() => {
  if (!linkedinUrl.value.trim()) return ''
  return validateHttpsUrl(linkedinUrl.value) ? '' : t('profile.errors.invalidHttpsUrl')
})

const githubError = computed(() => {
  if (!githubUrl.value.trim()) return ''
  return validateHttpsUrl(githubUrl.value) ? '' : t('profile.errors.invalidHttpsUrl')
})

const portfolioError = computed(() => {
  if (!portfolioUrl.value.trim()) return ''
  return validateHttpsUrl(portfolioUrl.value) ? '' : t('profile.errors.invalidHttpsUrl')
})

const targetRoleError = computed(() => {
  return targetRole.value.trim().length <= 120 ? '' : t('profile.errors.targetRoleTooLong')
})

// Overlap validation between required and excluded keywords (case-folded)
const keywordOverlapError = computed(() => {
  const reqFolded = new Set(requiredKeywords.value.map((k) => k.trim().toLowerCase()))
  for (const ex of excludedKeywords.value) {
    if (reqFolded.has(ex.trim().toLowerCase())) {
      return t('profile.errors.keywordOverlap', { keyword: ex })
    }
  }
  return ''
})

const isValid = computed(() => {
  return (
    !phoneError.value &&
    !locationError.value &&
    !linkedinError.value &&
    !githubError.value &&
    !portfolioError.value &&
    !targetRoleError.value &&
    !keywordOverlapError.value
  )
})

const addTargetLocation = () => {
  const val = newLocationInput.value.trim()
  if (!val || val.length > 80 || targetLocations.value.length >= 20) return
  const folded = val.toLowerCase()
  if (!targetLocations.value.some((item) => item.toLowerCase() === folded)) {
    targetLocations.value.push(val)
  }
  newLocationInput.value = ''
}

const removeTargetLocation = (idx: number) => {
  targetLocations.value.splice(idx, 1)
}

const addRequiredKeyword = () => {
  const val = newRequiredInput.value.trim()
  if (!val || val.length > 80 || requiredKeywords.value.length >= 20) return
  const folded = val.toLowerCase()
  if (!requiredKeywords.value.some((item) => item.toLowerCase() === folded)) {
    requiredKeywords.value.push(val)
  }
  newRequiredInput.value = ''
}

const removeRequiredKeyword = (idx: number) => {
  requiredKeywords.value.splice(idx, 1)
}

const addExcludedKeyword = () => {
  const val = newExcludedInput.value.trim()
  if (!val || val.length > 80 || excludedKeywords.value.length >= 20) return
  const folded = val.toLowerCase()
  if (!excludedKeywords.value.some((item) => item.toLowerCase() === folded)) {
    excludedKeywords.value.push(val)
  }
  newExcludedInput.value = ''
}

const removeExcludedKeyword = (idx: number) => {
  excludedKeywords.value.splice(idx, 1)
}

const loadProfile = async () => {
  isLoading.value = true
  loadError.value = ''
  try {
    const [identity, profile] = await Promise.all([getUserProfile(), getCandidateProfile()])
    identityName.value = identity.name || ''
    identityEmail.value = identity.email || ''
    identityRole.value = identity.role || ''

    phone.value = profile.phone || ''
    location.value = profile.location || ''
    linkedinUrl.value = profile.linkedin_url || ''
    githubUrl.value = profile.github_url || ''
    portfolioUrl.value = profile.portfolio_url || ''
    targetRole.value = profile.target_role || ''

    targetLocations.value = [...profile.target_locations]
    requiredKeywords.value = [...profile.required_keywords]
    excludedKeywords.value = [...profile.excluded_keywords]
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { data?: { error?: { message?: string } }; message?: string }
      loadError.value = apiErr.data?.error?.message || apiErr.message || t('profile.loadError')
    } else {
      loadError.value = t('profile.loadError')
    }
  } finally {
    isLoading.value = false
  }
}

const handleSave = async () => {
  if (!isValid.value || isSaving.value) return

  isSaving.value = true
  saveError.value = ''
  saveSuccess.value = false

  const payload: UpdateCandidateProfileRequestDto = {
    phone: phone.value.trim() || null,
    location: location.value.trim() || null,
    linkedin_url: linkedinUrl.value.trim() || null,
    github_url: githubUrl.value.trim() || null,
    portfolio_url: portfolioUrl.value.trim() || null,
    target_role: targetRole.value.trim() || null,
    required_keywords: requiredKeywords.value,
    excluded_keywords: excludedKeywords.value,
    target_locations: targetLocations.value,
  }

  try {
    await updateCandidateProfile(payload)
    saveSuccess.value = true
    setTimeout(() => {
      saveSuccess.value = false
    }, 4000)
  } catch (err: unknown) {
    if (err && typeof err === 'object') {
      const apiErr = err as { data?: { error?: { message?: string } }; message?: string }
      saveError.value = apiErr.data?.error?.message || apiErr.message || t('profile.saveError')
    } else {
      saveError.value = t('profile.saveError')
    }
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  loadProfile()
})
</script>

<template>
  <div class="max-w-4xl mx-auto py-8 px-4 sm:px-6 space-y-8">
    <!-- Header -->
    <div class="border-b border-slate-200 dark:border-slate-800 pb-5">
      <div class="flex items-center gap-2 mb-2">
        <span class="px-2.5 py-0.5 text-xs font-mono font-semibold bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded">
          {{ $t('profile.badge') }}
        </span>
      </div>
      <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ $t('profile.title') }}
      </h1>
      <p class="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl">
        {{ $t('profile.description') }}
      </p>
    </div>

    <!-- Loading State -->
    <div
      v-if="isLoading"
      class="p-12 border border-slate-200 dark:border-slate-800 rounded-lg flex flex-col items-center justify-center text-slate-500 gap-2 bg-white dark:bg-slate-900"
    >
      <div class="animate-spin rounded-full h-5 w-5 border-2 border-indigo-600 border-t-transparent" />
      <span class="text-xs">{{ $t('profile.loading') }}</span>
    </div>

    <!-- Load Error -->
    <div
      v-else-if="loadError"
      role="alert"
      class="p-4 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between"
    >
      <span>{{ loadError }}</span>
      <button
        type="button"
        @click="loadProfile"
        class="ml-3 px-2.5 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium transition"
      >
        {{ $t('common.retry') }}
      </button>
    </div>

    <!-- Form Content -->
    <form v-else @submit.prevent="handleSave" class="space-y-8">
      <!-- Section 1: Read-Only Identity -->
      <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <User class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
            <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white">
              {{ $t('profile.identityTitle') }}
            </h2>
          </div>
          <span class="inline-flex items-center gap-1 text-[11px] font-mono text-slate-400">
            <Lock class="w-3 h-3" />
            {{ $t('profile.readOnlyIdentity') }}
          </span>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <AppInput
            :model-value="identityName"
            :label="$t('profile.fullName')"
            disabled
          />
          <AppInput
            :model-value="identityEmail"
            :label="$t('profile.email')"
            disabled
          />
        </div>
      </div>

      <!-- Section 2: Contact Details -->
      <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-4">
        <div class="flex items-center gap-2">
          <Phone class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white">
            {{ $t('profile.contactTitle') }}
          </h2>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <AppInput
            v-model="phone"
            id="profile-phone"
            :label="$t('profile.phone')"
            :placeholder="$t('profile.phonePlaceholder')"
            :error="phoneError"
            :hint="$t('profile.phoneHint')"
          />
          <AppInput
            v-model="location"
            id="profile-location"
            :label="$t('profile.location')"
            :placeholder="$t('profile.locationPlaceholder')"
            :error="locationError"
          />
        </div>

        <div class="space-y-4 pt-2">
          <AppInput
            v-model="linkedinUrl"
            id="profile-linkedin"
            :label="$t('profile.linkedinUrl')"
            placeholder="https://linkedin.com/in/username"
            :error="linkedinError"
          />
          <AppInput
            v-model="githubUrl"
            id="profile-github"
            :label="$t('profile.githubUrl')"
            placeholder="https://github.com/username"
            :error="githubError"
          />
          <AppInput
            v-model="portfolioUrl"
            id="profile-portfolio"
            :label="$t('profile.portfolioUrl')"
            placeholder="https://example.com"
            :error="portfolioError"
          />
        </div>
      </div>

      <!-- Section 3: Job Targets & Keywords -->
      <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-xs space-y-6">
        <div class="flex items-center gap-2">
          <Briefcase class="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          <h2 class="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white">
            {{ $t('profile.targetsTitle') }}
          </h2>
        </div>

        <AppInput
          v-model="targetRole"
          id="profile-target-role"
          :label="$t('profile.targetRole')"
          :placeholder="$t('profile.targetRolePlaceholder')"
          :error="targetRoleError"
        />

        <!-- Target Locations List -->
        <div class="space-y-2">
          <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
            {{ $t('profile.targetLocations') }}
            <span class="text-[11px] font-normal text-slate-400">({{ targetLocations.length }}/20)</span>
          </label>
          <div class="flex items-center gap-2">
            <input
              v-model="newLocationInput"
              type="text"
              maxlength="80"
              :placeholder="$t('profile.addLocationPlaceholder')"
              @keydown.enter.prevent="addTargetLocation"
              class="flex-1 rounded-md border text-xs px-3 py-1.5 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 placeholder-slate-400 border-slate-300 dark:border-slate-700 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            />
            <AppButton
              type="button"
              variant="secondary"
              size="xs"
              @click="addTargetLocation"
              :disabled="!newLocationInput.trim() || targetLocations.length >= 20"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>{{ $t('common.add') }}</span>
            </AppButton>
          </div>
          <div v-if="targetLocations.length > 0" class="flex flex-wrap gap-1.5 pt-1">
            <span
              v-for="(loc, idx) in targetLocations"
              :key="idx"
              class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-slate-700"
            >
              <span>{{ loc }}</span>
              <button
                type="button"
                @click="removeTargetLocation(idx)"
                class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                :aria-label="`Remove ${loc}`"
              >
                <X class="w-3 h-3" />
              </button>
            </span>
          </div>
        </div>

        <!-- Required Keywords List -->
        <div class="space-y-2">
          <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
            {{ $t('profile.requiredKeywords') }}
            <span class="text-[11px] font-normal text-slate-400">({{ requiredKeywords.length }}/20)</span>
          </label>
          <div class="flex items-center gap-2">
            <input
              v-model="newRequiredInput"
              type="text"
              maxlength="80"
              :placeholder="$t('profile.addKeywordPlaceholder')"
              @keydown.enter.prevent="addRequiredKeyword"
              class="flex-1 rounded-md border text-xs px-3 py-1.5 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 placeholder-slate-400 border-slate-300 dark:border-slate-700 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            />
            <AppButton
              type="button"
              variant="secondary"
              size="xs"
              @click="addRequiredKeyword"
              :disabled="!newRequiredInput.trim() || requiredKeywords.length >= 20"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>{{ $t('common.add') }}</span>
            </AppButton>
          </div>
          <div v-if="requiredKeywords.length > 0" class="flex flex-wrap gap-1.5 pt-1">
            <span
              v-for="(kw, idx) in requiredKeywords"
              :key="idx"
              class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800"
            >
              <span>{{ kw }}</span>
              <button
                type="button"
                @click="removeRequiredKeyword(idx)"
                class="text-emerald-500 hover:text-emerald-700 dark:hover:text-emerald-200"
                :aria-label="`Remove ${kw}`"
              >
                <X class="w-3 h-3" />
              </button>
            </span>
          </div>
        </div>

        <!-- Excluded Keywords List -->
        <div class="space-y-2">
          <label class="block text-xs font-medium text-slate-700 dark:text-slate-300">
            {{ $t('profile.excludedKeywords') }}
            <span class="text-[11px] font-normal text-slate-400">({{ excludedKeywords.length }}/20)</span>
          </label>
          <div class="flex items-center gap-2">
            <input
              v-model="newExcludedInput"
              type="text"
              maxlength="80"
              :placeholder="$t('profile.addExcludedPlaceholder')"
              @keydown.enter.prevent="addExcludedKeyword"
              class="flex-1 rounded-md border text-xs px-3 py-1.5 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 placeholder-slate-400 border-slate-300 dark:border-slate-700 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            />
            <AppButton
              type="button"
              variant="secondary"
              size="xs"
              @click="addExcludedKeyword"
              :disabled="!newExcludedInput.trim() || excludedKeywords.length >= 20"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>{{ $t('common.add') }}</span>
            </AppButton>
          </div>
          <div v-if="excludedKeywords.length > 0" class="flex flex-wrap gap-1.5 pt-1">
            <span
              v-for="(kw, idx) in excludedKeywords"
              :key="idx"
              class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 border border-rose-200 dark:border-rose-800"
            >
              <span>{{ kw }}</span>
              <button
                type="button"
                @click="removeExcludedKeyword(idx)"
                class="text-rose-500 hover:text-rose-700 dark:hover:text-rose-200"
                :aria-label="`Remove ${kw}`"
              >
                <X class="w-3 h-3" />
              </button>
            </span>
          </div>
        </div>

        <!-- Overlap Alert -->
        <div
          v-if="keywordOverlapError"
          role="alert"
          class="p-3 rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center gap-2"
        >
          <AlertCircle class="w-4 h-4 shrink-0" />
          <span>{{ keywordOverlapError }}</span>
        </div>
      </div>

      <!-- Feedback Messages -->
      <div
        v-if="saveSuccess"
        class="p-3 rounded-md bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2"
      >
        <CheckCircle2 class="w-4 h-4 shrink-0" />
        <span>{{ $t('profile.saveSuccess') }}</span>
      </div>

      <div
        v-if="saveError"
        role="alert"
        class="p-3 rounded-md bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center gap-2"
      >
        <AlertCircle class="w-4 h-4 shrink-0" />
        <span>{{ saveError }}</span>
      </div>

      <!-- Submit Action -->
      <div class="flex items-center justify-end gap-3 pt-2">
        <AppButton
          type="submit"
          variant="primary"
          size="md"
          :loading="isSaving"
          :disabled="!isValid || isSaving"
        >
          {{ $t('common.save') }}
        </AppButton>
      </div>
    </form>
  </div>
</template>