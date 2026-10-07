export interface ConsoleLineDto {
  id: number
  raw: string
  timestamp?: string | null
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG' | 'RAW'
  thread?: string | null
  message: string
}

export interface ConsoleExportResultDto {
  success: boolean
  filePath?: string | null
  mclogsUrl?: string | null
  message: string
}

export type LogLevelFilter = 'ALL' | 'ERROR' | 'WARN' | 'INFO' | 'DEBUG'