import type { Component } from 'vue'

export interface AutoBuildRequestDto {
  prompt: string
  mcVersion: string
  loader: string
  maxMods: number
  includePerformance: boolean
  resolveKeybindsAfter: boolean
}

export interface BuildProgressDto {
  phase: 'AI_SYNTHESIS' | 'DEPENDENCY_GRAPH' | 'DOWNLOADING' | 'KEYBIND_OPTIMIZATION' | 'COMPLETED' | 'FAILED'
  percent: number
  currentStep: string
  currentMod?: string | null
  totalMods: number
  downloadedMods: number
  logLine?: string | null
}

export interface BuildResultDto {
  success: boolean
  totalInstalled: number
  installedMods: string[]
  keybindChanges: number
  message: string
}

export interface KeybindConflictDto {
  keyId: string
  oldBinding: string
  newBinding: string
}

export interface KeybindResolveReportDto {
  success: boolean
  conflictsResolved: number
  details: KeybindConflictDto[]
  message: string
}

export interface SynthPreset {
  id: string
  name: string
  desc: string
  icon: Component
  prompt: string
  loader: string
  maxMods: number
}