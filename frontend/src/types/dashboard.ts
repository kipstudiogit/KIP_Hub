export interface StorageBreakdownDto {
  totalSizeMb: number
  modsSizeMb: number
  savesSizeMb: number
  screenshotsSizeMb: number
  logsSizeMb: number
}

export interface RecentWorldOverviewDto {
  name: string
  lastPlayed: string
  mode: string
  hardcore: boolean
  icon: string
}

export interface DashboardOverviewDto {
  size: string
  saves: string
  playtime: string
  java: string
  modsCount: number
  ramUsagePercent: number
  totalRamGb: number
  usedRamGb: number
  cpuUsagePercent: number
  cpuBrand: string
  lastWorld?: RecentWorldOverviewDto | null
  latestScreenshotThumbnail?: string | null
  storage: StorageBreakdownDto
  isGameRunning: boolean
  runningGamePid?: number | null
  currentInstanceName: string
}