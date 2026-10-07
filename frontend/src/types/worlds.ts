export interface WorldCardDto {
  name: string
  seed: string
  mode: string
  hardcore: boolean
  difficulty: string
  dayCount: number
  mcVersion: string
  spawnX: number
  spawnY: number
  spawnZ: number
  sizeMb: number
  lastPlayed: string
  datapacks: number
  icon: string
  isLocked: boolean
  healing?: boolean
  syncing?: boolean
  cloning?: boolean
}

export interface VcsCommitDto {
  id: string
  message: string
  timestamp: string
  tree?: Record<string, string> | null
}

export interface WorldActionResultDto {
  success: boolean
  msg: string
}

export interface WorldMapResultDto {
  success: boolean
  image?: string | null
  spawnX: number
  spawnZ: number
  msg?: string | null
}