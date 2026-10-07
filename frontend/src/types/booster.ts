export interface SimdCapabilitiesDto {
  avx512Supported: boolean
  avx2Supported: boolean
  fmaSupported: boolean
  vectorWidthBits: number
  simdTier: string
}

export interface OffHeapArenaMetricsDto {
  totalAllocatedMb: number
  usedMemoryMb: number
  activeChunksInFlight: number
  allocationBumpsCount: number
  zeroGcEfficiencyPercent: number
}

export interface CheckerboardTopologyDto {
  currentPhase: number
  phaseName: string
  queuedChunks: number
  resolvedCascadesCount: number
  threadConcurrencyTier: string
}

export interface ChunkAcceleratorStatusDto {
  isActive: boolean
  simdCapabilities: SimdCapabilitiesDto
  ringBufferPendingChunks: number
  totalChunksBuffered: number
  totalBytesStreamedMb: number
  predictiveConeRadius: number
  estimatedTpsGain: number
  engineSignature: string
  arenaMetrics: OffHeapArenaMetricsDto
  topologyMetrics: CheckerboardTopologyDto
}

export interface ChunkPrebakeProgressDto {
  currentChunk: string
  generatedCount: number
  totalTargetChunks: number
  percent: number
  speedChunksPerSec: number
}

export interface ChunkPrebakeResultDto {
  success: boolean
  generatedChunks: number
  elapsedMs: number
  averageThroughput: number
  message: string
}

export interface ChunkRingFlushResultDto {
  chunksFlushed: number
  bytesWritten: number
  message: string
}

export interface MemoryMatrixStatusDto {
  largePagesSupported: boolean
  largePagesActive: boolean
  seLockPrivilegeGranted: boolean
  compactHeadersSupported: boolean
  compactHeadersActive: boolean
  standbyPrefaultFiles: number
  standbyPrefaultMb: number
  estimatedTlbMissReductionPercent: number
  savedMemoryMb: number
  statusText: string
}

export interface MemoryPrefaultResultDto {
  filesCached: number
  transferredMb: number
  message: string
}

export interface PingMasterConfigDto {
  routingMode: 'adaptive' | 'direct' | 'vpn' | 'socks5'
  upstreamSocks5Host: string
  upstreamSocks5Port: number
  antiBufferbloat: boolean
  dscpQosEnabled: boolean
}

export interface VpnDetectionDto {
  vpnActive: boolean
  activeAdapterName?: string | null
  adapterList: string[]
}

export interface PingMasterMetricDto {
  localProxyPort: number
  targetHost: string
  targetPort: number
  currentPingMs: number
  jitterMs: number
  minPingMs: number
  maxPingMs: number
  packetsOptimized: number
  bytesTransferred: number
  tcpNodelayActive: boolean
  bufferbloatReduced: boolean
  activeRoute: string
  vpnDetected: boolean
  activeVpnAdapter?: string | null
  routingMode: string
  hitRegQuality: string
  packetLossPercent: number
}

export interface PingMasterStatusDto {
  running: boolean
  localPort?: number | null
  targetHost: string
  targetPort: number
  message: string
  vpnDetected: boolean
  activeVpnAdapter?: string | null
}

export interface HardwareBoosterConfigDto {
  timerResolutionEnabled: boolean
  processPriorityBoost: boolean
  trimLauncherMemory: boolean
  defenderBypassEnabled: boolean
  cracAccelerationEnabled: boolean
  appCdsEnabled: boolean
  directVramTranscode: boolean
  shaderPrewarmingEnabled: boolean
  pageCacheWarmupEnabled: boolean
}

export interface HardwareBoosterStatusDto {
  timerActive: boolean
  timerResolutionMs: number
  trimmedMemoryMb: number
  defenderExcludedPaths: string[]
  appCdsArchiveExists: boolean
  appCdsArchiveSizeMb: number
  cracSupported: boolean
  cracCheckpointExists: boolean
  shaderCachePath: string
  pageCacheWarmFiles: number
  pageCacheWarmMb: number
}

export interface TranscodeProgressDto {
  currentFile: string
  processedCount: number
  totalFiles: number
  percent: number
  vramSavedMb: number
}

export interface TranscodeResultDto {
  success: boolean
  processedCount: number
  cachedEntries: number
  executionTimeMs: number
  message: string
}

export interface PageCacheWarmupResultDto {
  filesWarmed: number
  transferredMb: number
  message: string
}