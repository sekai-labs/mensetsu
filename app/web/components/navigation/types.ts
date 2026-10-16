import type { Component } from 'vue'
import {
  LayoutDashboard,
  Building2,
  FileText,
  CheckCircle2,
  GitFork,
  Network,
  BookOpen,
  HelpCircle,
  Code2,
  Quote,
  Settings,
} from 'lucide-vue-next'

export interface NavItem {
  id: string
  name: string
  nameKey?: string
  path: string
  icon: Component
  badge?: string
}

export interface NavSection {
  title: string
  titleKey?: string
  items: NavItem[]
}

export const WORKSPACE_NAV_SECTIONS: NavSection[] = [
  {
    title: 'Overview & Match',
    titleKey: 'navigation.sections.overviewAndMatch',
    items: [
      { id: 'overview', name: 'Overview', nameKey: 'navigation.items.overview', path: 'overview', icon: LayoutDashboard },
      { id: 'match', name: 'Match Analysis', nameKey: 'navigation.items.match', path: 'match', icon: CheckCircle2 },
      { id: 'skills', name: 'Skill Matrix', nameKey: 'navigation.items.skills', path: 'skills', icon: GitFork },
    ],
  },
  {
    title: 'Company & Role',
    titleKey: 'navigation.sections.companyAndRole',
    items: [
      { id: 'company', name: 'Company Dossier', nameKey: 'navigation.items.company', path: 'company', icon: Building2 },
      { id: 'role', name: 'Role & Job Description', nameKey: 'navigation.items.role', path: 'role', icon: FileText },
      { id: 'sources', name: 'Sources & Evidence', nameKey: 'navigation.items.sources', path: 'sources', icon: Quote },
    ],
  },
  {
    title: 'Preparation & Questions',
    titleKey: 'navigation.sections.preparationAndQuestions',
    items: [
      { id: 'learning', name: 'Study Plan', nameKey: 'navigation.items.learning', path: 'learning', icon: BookOpen },
      { id: 'questions', name: 'Question Bank', nameKey: 'navigation.items.questions', path: 'questions', icon: HelpCircle },
      { id: 'practice', name: 'Mock Practice', nameKey: 'navigation.items.practice', path: 'practice', icon: Code2 },
      { id: 'knowledge', name: 'Knowledge Graph', nameKey: 'navigation.items.knowledge', path: 'knowledge', icon: Network },
    ],
  },
  {
    title: 'Settings',
    titleKey: 'navigation.sections.settings',
    items: [
      { id: 'settings', name: 'Settings', nameKey: 'navigation.items.settings', path: 'settings', icon: Settings },
    ],
  },
]

export const ALL_WORKSPACE_NAV_ROUTES: string[] = WORKSPACE_NAV_SECTIONS.flatMap(
  (section) => section.items.map((item) => item.path)
)
