export interface WaypointItemDto {
  id: string
  name: string
  dimension: string
  x: number
  y: number
  z: number
  note: string
}

export type OverlayWidgetKey =
  | 'voice'
  | 'ai'
  | 'telemetry'
  | 'actions'
  | 'notes'
  | 'waypoints'
  | 'chunks'
  | 'pvp'
  | 'matrix'

export interface OverlayWidgetConfig {
  show: boolean
  pinned: boolean
  x: number
  y: number
  z: number
  width: number
  collapsed: boolean
}