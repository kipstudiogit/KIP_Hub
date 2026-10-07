export interface FriendDto {
  name: string
  avatar: string
  status: 'online' | 'in-game' | 'in-voice' | 'offline'
  activity?: string | null
  isFavorite: boolean
  note: string
  addedAt: string
}

export interface FriendActionDto {
  success: boolean
  msg: string
}

export interface PartyStateDto {
  active: boolean
  roomId: string
  hostEndpoint?: string | null
  membersCount: number
  installedModsCount: number
}