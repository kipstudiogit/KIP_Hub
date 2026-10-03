import { bridge } from '../bridge'
import { state } from '../stores/appState'

export function windowMinimize(): void {
  bridge.windowMinimize().catch(() => {})
}

export function windowMaximize(): void {
  bridge.windowMaximize().catch(() => {})
}

export function windowClose(): void {
  bridge.windowClose().catch(() => {})
}

export function toggleBigPicture(): void {
  state.isBigPicture = !state.isBigPicture
  bridge.toggleBigPicture().catch(() => {})
}

export function toggleMiniMode(): void {
  state.isMiniMode = !state.isMiniMode
  bridge.setMiniMode(state.isMiniMode).catch(() => {})
}