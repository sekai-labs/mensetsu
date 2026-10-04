<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useInterviewService } from '../services/interview.service'
import type { RoleRequirement } from '../models/interview.model'

const route = useRoute()
const workspaceId = computed(() => String(route.params.id))
const { getWorkspace, getWorkspaceMatch, getWorkspaceRole } = useInterviewService()

const { data: workspace } = await useAsyncData(
  `role-ws-${workspaceId.value}`,
  () => getWorkspace(workspaceId.value)
)
const { data: matchData } = await useAsyncData(
  `role-match-${workspaceId.value}`,
  () => getWorkspaceMatch(workspaceId.value)
)
const { data: roleData } = await useAsyncData(
  `role-jd-${workspaceId.value}`,
  () => getWorkspaceRole(workspaceId.value)
)

const mandatoryReqs = computed<RoleRequirement[]>(() => {
  const jdMandatory = roleData.value?.requirements?.filter((r) => r.importance === 'mandatory') || []
  if (jdMandatory.length > 0) return jdMandatory

  return (
    matchData.value?.gaps
      ?.filter((g) => g.importance === 'mandatory')
      .map((g) => ({
        skill: g.skill,
        requirement_quote: g.requirement_quote,
        importance: g.importance,
      })) ?? []
  )
})

const preferredReqs = computed<RoleRequirement[]>(() => {
  const jdPreferred = roleData.value?.requirements?.filter((r) => r.importance === 'preferred') || []
  if (jdPreferred.length > 0) return jdPreferred

  return (
    matchData.value?.gaps
      ?.filter((g) => g.importance === 'preferred')
      .map((g) => ({
        skill: g.skill,
        requirement_quote: g.requirement_quote,
        importance: g.importance,
      })) ?? []
  )
})
</script>

<template>
  <div class="space-y-6">
    <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-2">
      <div class="flex items-center gap-2 text-xs text-indigo-600 dark:text-indigo-400 font-semibold">
        <span>Target Position</span>
        <span>•</span>
        <span>{{ roleData?.company_name || workspace?.company_name }}</span>
      </div>
      <h1 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">
        {{ roleData?.title || workspace?.role_title }}
      </h1>
      <p class="text-xs text-slate-600 dark:text-slate-300 max-w-3xl">
        {{ roleData?.role_summary || `Target engineering position requiring hands-on technical architecture and distributed systems mastery at ${workspace?.company_name}.` }}
      </p>
    </div>

    <div v-if="roleData?.responsibilities?.length" class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-3">
      <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Role Responsibilities & Day-to-Day Scope</h2>
      <ul class="space-y-1.5 list-disc pl-5 text-xs text-slate-700 dark:text-slate-300">
        <li v-for="(resp, idx) in roleData.responsibilities" :key="idx">
          {{ resp }}
        </li>
      </ul>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
          <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Core / Mandatory Expectations</h2>
          <span class="text-xs px-2 py-0.5 rounded bg-rose-50 text-rose-700 font-mono font-semibold">
            {{ mandatoryReqs.length + (matchData?.strengths?.length ?? 0) }} Total
          </span>
        </div>

        <div class="space-y-3">
          <div
            v-for="(s, idx) in matchData?.strengths"
            :key="`mand-str-${idx}`"
            class="p-4 rounded-md border border-emerald-100 dark:border-emerald-950/60 bg-emerald-50/30 dark:bg-emerald-950/10"
          >
            <div class="flex items-center justify-between">
              <span class="font-semibold text-slate-900 dark:text-white text-xs">{{ s.skill }}</span>
              <span class="text-[10px] text-emerald-600 font-semibold uppercase tracking-wider">Evidenced</span>
            </div>
            <p class="text-xs text-slate-600 dark:text-slate-400 mt-1">{{ s.relevance }}</p>
          </div>

          <div
            v-for="(m, idx) in mandatoryReqs"
            :key="`mand-gap-${idx}`"
            class="p-4 rounded-md border border-amber-100 dark:border-amber-950/60 bg-amber-50/30 dark:bg-amber-950/10"
          >
            <div class="flex items-center justify-between">
              <span class="font-semibold text-slate-900 dark:text-white text-xs">{{ m.skill }}</span>
              <span class="text-[10px] text-amber-600 font-semibold uppercase tracking-wider">Gap / Focus</span>
            </div>
            <p class="text-xs text-slate-600 dark:text-slate-400 mt-1">{{ m.requirement_quote }}</p>
          </div>
        </div>
      </div>

      <div class="p-6 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm space-y-4">
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
          <h2 class="text-sm font-semibold text-slate-900 dark:text-white">Preferred Qualifications</h2>
          <span class="text-xs px-2 py-0.5 rounded bg-slate-100 text-slate-600 font-mono font-semibold">
            {{ preferredReqs.length }} Desired
          </span>
        </div>

        <div v-if="preferredReqs.length === 0" class="text-xs text-slate-500">
          No secondary preferred requirements identified.
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(p, idx) in preferredReqs"
            :key="idx"
            class="p-4 rounded-md border border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-850"
          >
            <div class="font-semibold text-slate-900 dark:text-white text-xs">{{ p.skill }}</div>
            <p class="text-xs text-slate-600 dark:text-slate-400 mt-1">{{ p.requirement_quote }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
