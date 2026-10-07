import type { Component } from 'vue'

export type StoreProvider = 'modrinth' | 'curseforge'
export type StoreProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader'

export interface StoreItemFileDto {
  filename: string
  url: string
  primary: boolean
  size: number
}

export interface StoreItemDependencyDto {
  projectId: string
  versionId?: string | null
  dependencyType: string
  fileName?: string | null
}

export interface StoreItemVersionDto {
  id: string
  versionNumber: string
  name: string
  date: string
  changelog: string
  files: StoreItemFileDto[]
  dependencies: StoreItemDependencyDto[]
  gameVersions: string[]
  loaders: string[]
}

export interface StoreItemGalleryDto {
  url: string
  title?: string | null
}

export interface StoreItemDetailsDto {
  body: string
  gallery: StoreItemGalleryDto[]
}

export interface StoreItemRecordDto {
  projectId: string
  slug: string
  title: string
  author: string
  description: string
  iconUrl: string
  downloads: number
  follows: number
  categories: string[]
  provider: StoreProvider
  projectType: StoreProjectType
  isInstalled: boolean
  installedFilename?: string | null
  downloading?: boolean
  progress?: number
  statusText?: string
}

export interface StoreSearchResultDto {
  success: boolean
  hits: StoreItemRecordDto[]
  totalHits: number
  msg?: string | null
}

export interface StoreDetailsResponseDto {
  success: boolean
  details: StoreItemDetailsDto
  versions: StoreItemVersionDto[]
  msg?: string | null
}

export interface StoreDownloadProgressDto {
  projectId: string
  filename: string
  progress: number
  status: string
}

export interface StoreInstallResultDto {
  success: boolean
  filename: string
  installedDependencies: string[]
  message: string
}

export interface StoreCategoryBadge {
  id: string
  name: string
  icon: Component
}

export interface StoreFilterOption {
  value: string
  label: string
}