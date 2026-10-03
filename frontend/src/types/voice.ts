export interface VoiceParticipant {
  id: string
  name: string
  speaking: boolean
  muted: boolean
  deafened: boolean
}

export interface VoiceState {
  isConnected: boolean
  channelId: string
  isMuted: boolean
  isDeafened: boolean
  localSpeaking: boolean
  participants: VoiceParticipant[]
  inputDevices: MediaDeviceInfo[]
  outputDevices: MediaDeviceInfo[]
  selectedInputId: string
  selectedOutputId: string
  showSettings: boolean
  isTestingMic: boolean
  testMicVolume: number
}

export interface WebRTCPeer {
  pc: RTCPeerConnection
  analyser: AnalyserNode | null
  audioCtx: AudioContext | null
  audioEl: HTMLAudioElement | null
  name: string
  makingOffer: boolean
  ignoreOffer: boolean
  isSettingRemoteAnswerPending: boolean
  pendingCandidates: RTCIceCandidateInit[]
  lastSpokeTime: number
  polite: boolean
}