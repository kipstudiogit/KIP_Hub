import { ref, reactive, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { WaypointItemDto, OverlayWidgetKey, OverlayWidgetConfig } from '../types/overlay'

const zIndexCounter = ref<number>(10000)
const isInteractive = ref<boolean>(false)

const widgets = reactive<Record<OverlayWidgetKey, OverlayWidgetConfig>>({
  chunks: { show: true, pinned: false, x: 30, y: 80, z: 10001, width: 340, collapsed: false },
  pvp: { show: true, pinned: false, x: 390, y: 80, z: 10002, width: 330, collapsed: false },
  matrix: { show: false, pinned: false, x: 740, y: 80, z: 10003, width: 320, collapsed: false },
  telemetry: { show: true, pinned: false, x: 30, y: 470, z: 10004, width: 300, collapsed: false },
  voice: { show: false, pinned: false, x: 350, y: 470, z: 10005, width: 320, collapsed: false },
  ai: { show: false, pinned: false, x: 690, y: 470, z: 10006, width: 380, collapsed: false },
  waypoints: { show: false, pinned: false, x: 1040, y: 80, z: 10007, width: 320, collapsed: false },
  actions: { show: false, pinned: false, x: 1090, y: 470, z: 10008, width: 260, collapsed: false },
  notes: { show: false, pinned: false, x: 740, y: 280, z: 10009, width: 320, collapsed: false },
})

const waypoints = ref<WaypointItemDto[]>([])
const telemetryHistory = ref<number[]>([35, 42, 38, 45, 52, 48, 50, 47, 55, 49])

const hasPinnedWidgets = computed<boolean>(() => {
  return Object.values(widgets).some((w) => w.show && w.pinned)
})

function bringToFront(id: OverlayWidgetKey): void {
  zIndexCounter.value += 1
  widgets[id].z = zIndexCounter.value
}

function toggleWidget(id: OverlayWidgetKey): void {
  widgets[id].show = !widgets[id].show
  if (widgets[id].show) {
    bringToFront(id)
  }
  saveLayout()
}

function togglePinWidget(id: OverlayWidgetKey): void {
  widgets[id].pinned = !widgets[id].pinned
  saveLayout()
}

function toggleCollapseWidget(id: OverlayWidgetKey): void {
  widgets[id].collapsed = !widgets[id].collapsed
  saveLayout()
}

async function updateWindowMode(active: boolean, forceExit = false): Promise<void> {
  isInteractive.value = active
  const pinned = !forceExit && hasPinnedWidgets.value
  try {
    await invoke('overlay_set_active', {
      active,
      hasPinned: pinned,
    })
  } catch {}
}

async function enterOverlayMode(): Promise<void> {
  state.isOverlayActive = true
  isInteractive.value = true
  document.body.classList.add('in-game-overlay')
  document.documentElement.classList.add('in-game-overlay')
  try {
    await invoke('overlay_set_active', {
      active: true,
      hasPinned: hasPinnedWidgets.value,
    })
  } catch {}
}

async function exitOverlayMode(): Promise<void> {
  state.isOverlayActive = false
  isInteractive.value = false
  document.body.classList.remove('in-game-overlay')
  document.documentElement.classList.remove('in-game-overlay')
  try {
    await invoke('overlay_set_active', {
      active: false,
      hasPinned: false,
    })
  } catch {}
}

async function toggleOverlayState(): Promise<void> {
  if (!state.isOverlayActive) {
    await enterOverlayMode()
  } else if (!isInteractive.value) {
    await updateWindowMode(true)
  } else {
    if (hasPinnedWidgets.value) {
      await updateWindowMode(false)
    } else {
      await exitOverlayMode()
    }
  }
}

async function saveLayout(): Promise<void> {
  try {
    await invoke('overlay_save_layout', { layout: widgets })
  } catch {}
}

async function loadLayout(): Promise<void> {
  try {
    const stored = await invoke<Record<string, Partial<OverlayWidgetConfig>>>('overlay_load_layout')
    if (stored && Object.keys(stored).length > 0) {
      Object.entries(stored).forEach(([key, val]) => {
        const k = key as OverlayWidgetKey
        if (widgets[k]) {
          if (typeof val.show === 'boolean') widgets[k].show = val.show
          if (typeof val.pinned === 'boolean') widgets[k].pinned = val.pinned
          if (typeof val.collapsed === 'boolean') widgets[k].collapsed = val.collapsed
          if (typeof val.x === 'number') widgets[k].x = val.x
          if (typeof val.y === 'number') widgets[k].y = val.y
          if (typeof val.z === 'number') widgets[k].z = val.z
        }
      })
    }
  } catch {}
}

async function loadWaypoints(): Promise<void> {
  try {
    waypoints.value = await invoke<WaypointItemDto[]>('overlay_load_waypoints')
  } catch {
    waypoints.value = []
  }
}

async function saveWaypoints(): Promise<void> {
  try {
    await invoke('overlay_save_waypoints', { waypoints: waypoints.value })
  } catch {}
}

function addWaypoint(name: string, dimension: string, x: number, y: number, z: number, note = ''): void {
  const id = `wp_${Date.now().toString(36)}`
  waypoints.value.unshift({
    id,
    name: name.trim() || 'POI Location',
    dimension,
    x,
    y,
    z,
    note: note.trim(),
  })
  saveWaypoints()
  showToast(t('Waypoint Saved'), `${name} [${x}, ${y}, ${z}]`, 'success')
}

function removeWaypoint(id: string): void {
  waypoints.value = waypoints.value.filter((w) => w.id !== id)
  saveWaypoints()
}

async function copyWaypointTp(wp: WaypointItemDto): Promise<void> {
  const cmd = `/execute in minecraft:${wp.dimension.toLowerCase()} run tp @s ${wp.x} ${wp.y} ${wp.z}`
  await navigator.clipboard.writeText(cmd)
  showToast(t('Copied'), cmd, 'success')
}

function pushTelemetryPoint(val: number): void {
  telemetryHistory.value.push(val)
  if (telemetryHistory.value.length > 20) {
    telemetryHistory.value.shift()
  }
}

export function useOverlay() {
  return {
    widgets,
    waypoints,
    telemetryHistory,
    isInteractive,
    hasPinnedWidgets,
    bringToFront,
    toggleWidget,
    togglePinWidget,
    toggleCollapseWidget,
    updateWindowMode,
    enterOverlayMode,
    exitOverlayMode,
    toggleOverlayState,
    loadLayout,
    saveLayout,
    loadWaypoints,
    addWaypoint,
    removeWaypoint,
    copyWaypointTp,
    pushTelemetryPoint,
  }
}