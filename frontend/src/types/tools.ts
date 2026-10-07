import type { Component } from 'vue'

export interface ToolItem {
  id: string
  name: string
  desc: string
  bgClass: string
  textClass: string
  shadowClass: string
  icon: Component
}

export interface ToolExecutionResultDto {
  success: boolean
  msg: string
  clipboard?: string | null
}

export interface SafeModeStatusDto {
  enabled: boolean
}

export interface BytecodeStatsDto {
  java_8: number
  java_17: number
  java_21: number
  java_25: number
  max_detected_major: number
}

export interface DoctorIssueDto {
  id: string
  type: 'CRITICAL' | 'ERROR' | 'WARNING' | 'OPTIMIZATION'
  category: string
  title: string
  text: string
  action: string
  target_file: string
  target_slug: string
  extra_info: string
  selected?: boolean
}

export interface DoctorAnalysisReportDto {
  is_clean: boolean
  health_score: number
  risk_level: string
  total_checked: number
  target_mc: string
  target_loader: string
  bytecode_stats: BytecodeStatsDto
  issues: DoctorIssueDto[]
}

export interface ThreatIndicatorDto {
  category: string
  title: string
  severity: string
  weight: number
  location: string
}

export interface FileSecurityReportDto {
  filepath: string
  filename: string
  sha256: string
  threat_score: number
  threat_level: string
  is_clean: boolean
  entropy: number
  indicators: ThreatIndicatorDto[]
  file_size: number
}

export interface ShieldScanReportDto {
  total_scanned: number
  clean_count: number
  threat_count: number
  critical_count: number
  threats: FileSecurityReportDto[]
  timestamp: string
}

export interface ShieldScanProgressDto {
  currentFile: string
  scannedCount: number
  totalFiles: number
  percent: number
  threatsFound: number
}

export interface QuarantineRecordDto {
  id: string
  original_path: string
  filename: string
  isolated_filename: string
  threat_name: string
  threat_score: number
  sha256: string
  quarantined_at: string
  size_bytes: number
}

export interface CrashInvestigationDto {
  hasCrash: boolean
  filename: string
  timestamp: string
  culpritMod?: string | null
  hints: string[]
  rawSnippet: string
}