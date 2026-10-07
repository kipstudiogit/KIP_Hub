import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { state } from '../stores/appState'

export interface WindowStatusDto {
  isMaximized: boolean
  isMinimized: boolean
  isFullscreen: boolean
  isAlwaysOnTop: boolean
  width: number
  height: number
  scaleFactor: number
}

export interface MiniModeResultDto {
  isMini: boolean
  width: number
  height: number
}

export const isMaximized = ref<boolean>(false)
export const isPinned = ref<boolean>(false)

export async function syncWindowState(): Promise<void> {
  try {
    const status = await invoke<WindowStatusDto>('window_get_status')
    if (status) {
      isMaximized.value = status.isMaximized
    }
  } catch {
    isMaximized.value = false
  }
}

export async function minimizeWindow(): Promise<void> {
  try {
    await invoke('window_minimize')
  } catch {}
}

export async function toggleMaximize(): Promise<void> {
  try {
    const nextState = await invoke<boolean>('window_toggle_maximize')
    isMaximized.value = nextState
  } catch {
    isMaximized.value = !isMaximized.value
  }
}

export async function closeWindow(): Promise<void> {
  try {
    await invoke('window_close')
  } catch {}
}

export async function togglePinWindow(): Promise<void> {
  try {
    const nextPin = await invoke<boolean>('window_toggle_pin', { currentPinned: isPinned.value })
    isPinned.value = nextPin
  } catch {
    isPinned.value = !isPinned.value
  }
}

export async function toggleFullscreen(): Promise<void> {
  try {
    await invoke<boolean>('window_toggle_fullscreen')
  } catch {}
}

export async function toggleMiniMode(): Promise<void> {
  state.isMiniMode = !state.isMiniMode
  try {
    await invoke<MiniModeResultDto>('window_set_mini_mode', { mini: state.isMiniMode })
  } catch {}
}

export function toggleBigPicture(): void {
  state.isBigPicture = !state.isBigPicture
  toggleFullscreen().catch(() => {})
}

export function windowMinimize(): void {
  minimizeWindow()
}

export function windowMaximize(): void {
  toggleMaximize()
}

export function windowClose(): void {
  closeWindow()
}