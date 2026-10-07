import type { Component } from 'vue'

export type LoaderType = 'vanilla' | 'fabric' | 'forge' | 'neoforge' | 'quilt'

export interface LaunchRequestDto {
  version: string
  loader: LoaderType
  loaderVersion?: string | null
  ramAllocation?: number | null
  jvmArgs?: string | null
  resolution?: string | null
  fullscreen: boolean
  bypassChecks: boolean
}

export interface LaunchResultDto {
  success: boolean
  message: string
  pid?: number | null
}

export interface LaunchProgressDto {
  phase: string
  percent: number
  message: string
}

export interface PreflightIssueDto {
  level: string
  title: string
  description: string
  autoFixable: boolean
}

export interface PreflightReportDto {
  readyToLaunch: boolean
  javaCompatible: boolean
  javaVersion: string
  javaPath: string
  issues: PreflightIssueDto[]
  memoryAllocatedGb: number
  totalSystemMemoryGb: number
  hasCriticalConflicts: boolean
}

export interface AutoRepairResultDto {
  success: boolean
  fixedCount: number
  message: string
}

export interface BackendAppError {
  kind: string
  message: string
}

export interface LaunchPreset {
  id: string
  name: string
  desc: string
  loader: LoaderType
  badge: string
  badgeClass: string
  bgClass: string
  icon: Component
}