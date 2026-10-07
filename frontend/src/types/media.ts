export interface MediaItemDto {
  filename: string
  thumbnail: string
  sizeMb: number
  isPng: boolean
  width: number
  height: number
  dateModified: string
}

export interface MediaCompressProgressDto {
  currentFile: string
  processedCount: number
  totalFiles: number
  savedMb: number
  percent: number
}

export interface MediaCompressResultDto {
  success: boolean
  compressedCount: number
  savedMb: number
  msg: string
}

export interface MediaBatchActionResultDto {
  success: boolean
  affectedCount: number
  msg: string
}

export type MediaFormatFilter = 'all' | 'png' | 'jpg'