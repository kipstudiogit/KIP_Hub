import type { Component } from 'vue'

export interface AppSettingsDto {
  lang: string
  autostart: boolean
  theme: string
  customColor: string
  appearance: string
  mica: boolean
  rpc: boolean
  scale: number
  instances: string[]
  currentInstance: string
  autoBackup: boolean
  pteroUrl: string
  lowGraphics: boolean
  aiProvider: string
  aiModel: string
  ollamaUrl: string
  closeOnLaunch: boolean
  ramAllocation: number
  jvmGc: string
  jvmPreset: string
  shieldAutoScan: boolean
  voiceNoiseSuppression: boolean
  eulaAccepted: boolean
  telemetryOptIn: boolean
  kipUsername: string
  offlineUsername: string
  gameResolution: string
  gameFullscreen: boolean
  customJavaPath: string
  customJvmArgs: string
  themeAccent: string
  aiApiKey: string
  openaiApiKey: string
  anthropicApiKey: string
  cfApiKey: string
  safeMode: boolean
}

export interface JavaValidationResultDto {
  valid: boolean
  versionStr: string
  major: number
  message: string
}

export interface DetectedJavaRuntimeDto {
  path: string
  major: number
  versionStr: string
  vendor: string
}

export interface AiTestResultDto {
  success: boolean
  latencyMs: number
  message: string
}

export interface SettingsTabItem {
  id: 'general' | 'display' | 'jvm' | 'ai' | 'storage'
  label: string
  icon: Component
}

export interface AccentOptionItem {
  id: string
  label: string
  class: string
}