export interface SystemTelemetryDto {
  osName: string
  osVersion: string
  cpuBrand: string
  cpuCores: number
  totalRamGb: number
  availableRamGb: number
  javaVersion: string
  appVersion: string
  activeInstance: string
}

export interface AiDiagnosticResponseDto {
  success: boolean
  provider: string
  model: string
  localHints: string[]
  culpritMod?: string | null
  analysis: string
}

export interface BugReportSubmissionDto {
  reportText: string
  includeTelemetry: boolean
}

export interface BugReportResultDto {
  success: boolean
  reportId: string
  message: string
}