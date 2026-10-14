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
  path: string
  icon: Component
  badge?: string
}

export interface NavSection {
  title: string
  items: NavItem[]
}

export const WORKSPACE_NAV_SECTIONS: NavSection[] = [
  {
    title: 'Core Dossier',
    items: [
      { id: 'overview', name: 'Overview', path: 'overview', icon: LayoutDashboard },
      { id: 'company', name: 'Company Dossier', path: 'company', icon: Building2 },
      { id: 'research', name: 'Deep Research', path: 'research', icon: Network },
      { id: 'role', name: 'Role & JD', path: 'role', icon: FileText },
      { id: 'match', name: 'Match Analysis', path: 'match', icon: CheckCircle2 },
    ],
  },
  {
    title: 'Deep Intelligence',
    items: [
      { id: 'skills', name: 'Skill Map', path: 'skills', icon: GitFork },
      { id: 'knowledge', name: 'Knowledge Graph', path: 'knowledge', icon: Network },
      { id: 'sources', name: 'Sources & Citations', path: 'sources', icon: Quote },
    ],
  },
  {
    title: 'Execution & Prep',
    items: [
      { id: 'learning', name: 'Learning Plan', path: 'learning', icon: BookOpen },
      { id: 'questions', name: 'Question Book', path: 'questions', icon: HelpCircle },
      { id: 'practice', name: 'Practice Cockpit', path: 'practice', icon: Code2 },
    ],
  },
  {
    title: 'Workspace Settings',
    items: [
      { id: 'settings', name: 'Workspace Settings', path: 'settings', icon: Settings },
    ],
  },
]

export const ALL_WORKSPACE_NAV_ROUTES: string[] = WORKSPACE_NAV_SECTIONS.flatMap(
  (section) => section.items.map((item) => item.path)
)
