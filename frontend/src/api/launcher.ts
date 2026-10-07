import { invoke, Channel } from '@tauri-apps/api/core'
import type {
  LaunchRequestDto,
  LaunchResultDto,
  LaunchProgressDto,
  PreflightReportDto,
  AutoRepairResultDto,
} from '../types/launcher'

export async function apiInspectPreflight(
  version: string,
  loader: string,
  checkMods: boolean
): Promise<PreflightReportDto> {
  return await invoke<PreflightReportDto>('inspect_preflight', {
    version,
    loader,
    checkMods,
  })
}

export async function apiAutoRepairEnvironment(
  version: string,
  loader: string
): Promise<AutoRepairResultDto> {
  return await invoke<AutoRepairResultDto>('auto_repair_environment', {
    version,
    loader,
  })
}

export async function apiLaunchInstanceStream(
  payload: LaunchRequestDto,
  onProgress: (progress: LaunchProgressDto) => void
): Promise<LaunchResultDto> {
  const channel = new Channel<LaunchProgressDto>()
  channel.onmessage = (message: LaunchProgressDto) => {
    onProgress(message)
  }

  return await invoke<LaunchResultDto>('launch_instance_stream', {
    payload,
    progressChannel: channel,
  })
}