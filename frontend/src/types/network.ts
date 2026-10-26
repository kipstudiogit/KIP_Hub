export type NetworkSubTab = 'voice' | 'radar' | 'remote' | 'skin'

export interface ServerPingResultDto {
  online: boolean
  ping?: number | null
  motd?: string | null
  players?: string | null
  icon?: string | null
}

export interface PteroServerDto {
  id: string
  name: string
  state: string
}

export interface NetworkActionResultDto {
  success: boolean
  msg: string
}

export interface PartyInvitePreparedDto {
  mods: string[]
  tunnelUrl: string
}

export interface DiscoveredLanServerDto {
  motd: string
  port: number
  address: string
  lastSeenEpoch: number
}

export type SkinAnimationType = 'idle' | 'walk' | 'run'
export type DockerCoreType = 'paper' | 'fabric' | 'forge' | 'vanilla'