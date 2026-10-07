import type { Component } from 'vue'

export type ContentTabType = 'mods' | 'modpacks' | 'resourcepacks' | 'shaderpacks'

export interface GraphNodeColor {
  background: string
  border: string
  highlight: {
    background: string
    border: string
  }
}

export interface GraphNodeFont {
  color: string
  size: number
  face: string
}

export interface GraphNode {
  id: string
  label: string
  title: string
  shape: string
  size: number
  color: GraphNodeColor
  font: GraphNodeFont
  version: string
  modType: string
}

export interface GraphEdgeColor {
  color: string
  highlight: string
}

export interface GraphEdge {
  from: string
  to: string
  color: GraphEdgeColor
  arrows: string
  dashes: boolean
  label?: string | null
}

export interface ModGraphDataDto {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface ModUpdateItemDto {
  filename: string
  projectId: string
  name: string
  currentVersion: string
  latestVersion: string
  downloadUrl: string
  newFilename: string
}

export interface ModUpdatesCheckDto {
  success: boolean
  updates: ModUpdateItemDto[]
  checkedCount: number
}

export interface ContentUpdateProgressDto {
  currentFile: string
  index: number
  total: number
  percent: number
  completed: boolean
}

export interface ContentActionResultDto {
  success: boolean
  msg: string
  count?: number | null
}

export interface ModpackRecordDto {
  filename: string
  name: string
  version: string
  author: string
  description: string
  mcVersion: string
  loader: string
  modCount: number
  icon: string
  sizeBytes: number
}

export interface HubPreset {
  id: string
  title: string
  author: string
  description: string
  preset: string[]
}

export interface SwarmStatusDto {
  name: string
  seeders: number
  peers: number
  upload_rate: number
  total_upload: number
}