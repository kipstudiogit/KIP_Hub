import { reactive, ref } from 'vue'
import {
  bridge,
  setupTauriListeners,
  type DashboardStats,
  type FriendRecord,
} from './bridge'

export interface NavItem {
  id: string
  label: string
  icon: string
}

export interface SettingsState {
  mc_dir: string
  has_ms_token: boolean
  ms_name: string
  has_kip_token: boolean
  kip_username: string
  auto_backup: boolean
  rpc: boolean
  ai_provider: string
  ai_api_key: string
  openai_api_key: string
  anthropic_api_key: string
  ollama_url: string
  cf_api_key: string
  lang: string
  autostart: boolean
  safe_mode: boolean
  low_graphics: boolean
  close_on_launch: boolean
  ram_allocation: number
  shield_auto_scan: boolean
  voice_noise_suppression: boolean
  eula_accepted: boolean
  telemetry_opt_in: boolean
  instances: string[]
  offline_username: string
  game_resolution: string
  game_fullscreen: boolean
  custom_java_path: string
  custom_jvm_args: string
}

export interface PartyInviteData {
  senderId: string
  senderName: string
  mods: string[]
  tunnelUrl: string
}

export interface AppState {
  appName: string
  appAccent: string
  greeting: string
  showBoot: boolean
  bootProgress: number
  bootText: string
  isOverlayActive: boolean
  isMiniMode: boolean
  isBigPicture: boolean
  isNexusOpen: boolean
  currentView: string
  partyInvite: PartyInviteData | null
  translations: Record<string, Record<string, string>>
  navItems: NavItem[]
  stats: DashboardStats
  isMcRunning: boolean
  mcStatusText: string
  mcVersions: string[]
  launchStatus: string
  launchProgress: number
  settings: SettingsState
  friends: FriendRecord[]
  newsText: string
  consoleHtml: string
  _consoleBuffer: string[]
  aiInputText: string
  version: string
}

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

export interface ToastItem {
  id: number
  title: string
  message: string
  type: 'info' | 'success' | 'danger'
  icon: string
}

interface WebRTCPeer {
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

export const api = ref(bridge)

export const state = reactive<AppState>({
  appName: 'K.I.P.',
  appAccent: ' Hub',
  greeting: 'Welcome',
  showBoot: true,
  bootProgress: 0,
  bootText: 'INITIALIZING NEURAL CORE...',
  isOverlayActive: false,
  isMiniMode: false,
  isBigPicture: false,
  isNexusOpen: false,
  currentView: 'dashboard',
  partyInvite: null,
  translations: {
    ru: {}, es: {}, de: {}, zh: {}, ja: {}, ko: {}, tr: {}, fr: {}, pt: {}, it: {}, pl: {}, en: {}
  },
  navItems: [
    { id: 'dashboard', label: 'Overview', icon: 'layout-dashboard' },
    { id: 'launcher', label: 'Launcher', icon: 'gamepad-2' },
    { id: 'builder', label: 'Auto-Builder', icon: 'wand-2' },
    { id: 'mods', label: 'Content', icon: 'puzzle' },
    { id: 'store', label: 'Store', icon: 'shopping-cart' },
    { id: 'worlds', label: 'Worlds', icon: 'globe' },
    { id: 'network', label: 'Network', icon: 'wifi' },
    { id: 'tools', label: 'Tools', icon: 'zap' },
    { id: 'media', label: 'Gallery', icon: 'image' },
    { id: 'console', label: 'Console', icon: 'terminal' },
    { id: 'support', label: 'Support', icon: 'life-buoy' },
    { id: 'settings', label: 'Settings', icon: 'settings' }
  ],
  stats: { size: '...', saves: '...', playtime: '...', java: '...' },
  isMcRunning: false,
  mcStatusText: 'Minecraft stopped',
  mcVersions: [],
  launchStatus: 'Ready',
  launchProgress: 0,
  settings: {
    mc_dir: '',
    has_ms_token: false,
    ms_name: '',
    has_kip_token: false,
    kip_username: '',
    auto_backup: false,
    rpc: true,
    ai_provider: 'google',
    ai_api_key: '',
    openai_api_key: '',
    anthropic_api_key: '',
    ollama_url: 'http://localhost:11434',
    cf_api_key: '',
    lang: 'en',
    autostart: false,
    safe_mode: false,
    low_graphics: false,
    close_on_launch: false,
    ram_allocation: 0,
    shield_auto_scan: true,
    voice_noise_suppression: true,
    eula_accepted: false,
    telemetry_opt_in: false,
    instances: [],
    offline_username: 'Player',
    game_resolution: '854x480',
    game_fullscreen: false,
    custom_java_path: '',
    custom_jvm_args: ''
  },
  friends: [],
  newsText: '',
  consoleHtml: 'Awaiting data stream...',
  _consoleBuffer: [],
  aiInputText: '',
  version: '1.5.8'
})

export const voiceState = reactive<VoiceState>({
  isConnected: false,
  channelId: '',
  isMuted: false,
  isDeafened: false,
  localSpeaking: false,
  participants: [],
  inputDevices: [],
  outputDevices: [],
  selectedInputId: localStorage.getItem('kip_mic_id') || 'default',
  selectedOutputId: localStorage.getItem('kip_speaker_id') || 'default',
  showSettings: false,
  isTestingMic: false,
  testMicVolume: 0
})

export const toasts = ref<ToastItem[]>([])
let toastIdCounter = 0

let rawMicStream: MediaStream | null = null
let localStream: MediaStream | null = null
let localDummyAudio: HTMLAudioElement | null = null
let audioContext: AudioContext | null = null
let localAnalyser: AnalyserNode | null = null
let expanderGainNode: GainNode | null = null
let adaptiveNoiseFloor = 10
let vadIntervalId: ReturnType<typeof setInterval> | null = null

let signalingSocket: WebSocket | null = null
let pingInterval: ReturnType<typeof setInterval> | null = null
let peers: Record<string, WebRTCPeer> = {}

let localLastSpokeTime = 0
let isDeviceListenerAdded = false

const RTC_CONFIG: RTCConfiguration = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
    { urls: 'stun:stun2.l.google.com:19302' },
    { urls: 'stun:stun3.l.google.com:19302' },
    { urls: 'stun:stun4.l.google.com:19302' },
    { urls: 'stun:stun.cloudflare.com:3478' },
    { urls: 'stun:stun.services.mozilla.com:3478' }
  ],
  iceCandidatePoolSize: 10
}

export function showToast(title: string, message: string, type: 'info' | 'success' | 'danger' = 'info'): void {
  const id = toastIdCounter++
  let icon = 'info'
  if (type === 'success') icon = 'check-circle'
  if (type === 'danger') icon = 'x-circle'

  toasts.value.push({ id, title, message, type, icon })
  if (toasts.value.length > 5) toasts.value.shift()

  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }, 4000)
}

export function t(key: string): string {
  if (!key) return ''
  const lang = state.settings.lang
  if (!lang || !state.translations[lang]) return key
  return state.translations[lang][key] || key
}

export function sanitizeHTML(str: string): string {
  const temp = document.createElement('div')
  temp.textContent = str
  return temp.innerHTML
}

export function getAvatarUrl(name: string): string {
  if (!name) return ''
  return `https://mc-heads.net/avatar/${encodeURIComponent(name)}/100`
}

export async function loadTranslations(): Promise<void> {
  const lang = state.settings.lang
  if (!lang) return
  if (Object.keys(state.translations[lang] || {}).length === 0) {
    try {
      const dict = await bridge.getTranslations(lang)
      if (dict && Object.keys(dict).length > 0) {
        state.translations[lang] = dict
      }
    } catch {
    }
  }
}

export async function loadSettings(): Promise<void> {
  try {
    const s = await bridge.getSettings()
    if (s) {
      state.settings.mc_dir = typeof s.mc_dir === 'string' ? s.mc_dir : ''
      state.settings.auto_backup = Boolean(s.auto_backup)
      state.settings.rpc = s.rpc !== false
      state.settings.ai_provider = typeof s.ai_provider === 'string' ? s.ai_provider : 'google'
      state.settings.ai_api_key = typeof s.ai_api_key === 'string' ? s.ai_api_key : ''
      state.settings.openai_api_key = typeof s.openai_api_key === 'string' ? s.openai_api_key : ''
      state.settings.anthropic_api_key = typeof s.anthropic_api_key === 'string' ? s.anthropic_api_key : ''
      state.settings.ollama_url = typeof s.ollama_url === 'string' ? s.ollama_url : 'http://localhost:11434'
      state.settings.cf_api_key = typeof s.cf_api_key === 'string' ? s.cf_api_key : ''
      state.settings.lang = typeof s.lang === 'string' ? s.lang : 'en'
      state.settings.autostart = Boolean(s.autostart)
      state.settings.safe_mode = Boolean(s.safe_mode)
      state.settings.low_graphics = Boolean(s.low_graphics)
      state.settings.close_on_launch = Boolean(s.close_on_launch)
      state.settings.ram_allocation = typeof s.ram_allocation === 'number' ? s.ram_allocation : 0
      state.settings.shield_auto_scan = s.shield_auto_scan !== false
      state.settings.voice_noise_suppression = s.voice_noise_suppression !== false
      state.settings.eula_accepted = Boolean(s.eula_accepted)
      state.settings.telemetry_opt_in = Boolean(s.telemetry_opt_in)
      state.settings.instances = Array.isArray(s.instances) ? (s.instances as string[]) : []
      state.settings.offline_username = typeof s.offline_username === 'string' ? s.offline_username : 'Player'
      state.settings.game_resolution = typeof s.game_resolution === 'string' ? s.game_resolution : '854x480'
      state.settings.game_fullscreen = Boolean(s.game_fullscreen)
      state.settings.custom_java_path = typeof s.custom_java_path === 'string' ? s.custom_java_path : ''
      state.settings.custom_jvm_args = typeof s.custom_jvm_args === 'string' ? s.custom_jvm_args : ''
    }

    const prof = await bridge.getMsProfile()
    if (prof && prof.success) {
      state.settings.has_ms_token = true
      state.settings.ms_name = prof.name || ''
    } else {
      state.settings.has_ms_token = false
      state.settings.ms_name = ''
    }

    const kipProf = await bridge.getKipProfile()
    if (kipProf && kipProf.success) {
      state.settings.has_kip_token = true
      state.settings.kip_username = kipProf.username || ''
    } else {
      state.settings.has_kip_token = false
      state.settings.kip_username = ''
    }
  } catch {
  }
}

export async function loadDashboardStats(): Promise<void> {
  try {
    const s = await bridge.getDashboardStats()
    if (s) {
      state.stats.size = s.size
      state.stats.saves = s.saves
      state.stats.playtime = s.playtime
      state.stats.java = s.java
    }
  } catch {
  }
}

export async function saveSetting(key: string, value: unknown): Promise<boolean> {
  try {
    (state.settings as Record<string, unknown>)[key] = value
    const result = await bridge.saveSetting(key, value)
    if (key === 'lang') {
      await loadTranslations()
    }
    return result
  } catch {
    return false
  }
}

export async function acceptEula(): Promise<void> {
  state.settings.eula_accepted = true
  await saveSetting('eula_accepted', true)
}

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

export async function loadAudioDevices(): Promise<void> {
  if (!navigator.mediaDevices?.enumerateDevices) return

  try {
    let tempStream: MediaStream | null = null
    try {
      tempStream = await navigator.mediaDevices.getUserMedia({ audio: true })
    } catch {
    }

    const devices = await navigator.mediaDevices.enumerateDevices()
    voiceState.inputDevices = devices.filter((d) => d.kind === 'audioinput')
    voiceState.outputDevices = devices.filter((d) => d.kind === 'audiooutput')

    if (tempStream) {
      tempStream.getTracks().forEach((t) => t.stop())
    }

    if (!isDeviceListenerAdded) {
      navigator.mediaDevices.addEventListener('devicechange', loadAudioDevices)
      isDeviceListenerAdded = true
    }
  } catch {
  }
}

function getValidDeviceId(savedId: string, devices: MediaDeviceInfo[]): string {
  if (!savedId || savedId === 'default') return 'default'
  const exists = devices.find((d) => d.deviceId === savedId)
  return exists ? savedId : 'default'
}

export async function toggleVoiceSettings(): Promise<void> {
  voiceState.showSettings = !voiceState.showSettings
  if (voiceState.showSettings) {
    await loadAudioDevices()
  } else {
    stopMicTest()
  }
}

async function setupDSPChain(): Promise<void> {
  if (rawMicStream) {
    rawMicStream.getTracks().forEach((t) => t.stop())
  }

  const validDeviceId = getValidDeviceId(voiceState.selectedInputId, voiceState.inputDevices)
  const constraints: MediaStreamConstraints = {
    audio: {
      channelCount: 1,
      sampleRate: 48000,
      echoCancellation: { ideal: true },
      noiseSuppression: { ideal: true },
      autoGainControl: { ideal: false }
    },
    video: false
  }

  if (validDeviceId !== 'default') {
    (constraints.audio as MediaTrackConstraints).deviceId = { exact: validDeviceId }
  }

  rawMicStream = await navigator.mediaDevices.getUserMedia(constraints)

  if (audioContext) {
    await audioContext.close().catch(() => {})
  }

  const AudioCtxClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext
  audioContext = new AudioCtxClass({ sampleRate: 48000, latencyHint: 'interactive' })
  await audioContext.resume()

  const source = audioContext.createMediaStreamSource(rawMicStream)
  const dest = audioContext.createMediaStreamDestination()

  localAnalyser = audioContext.createAnalyser()
  localAnalyser.fftSize = 512
  localAnalyser.smoothingTimeConstant = 0.2

  if (state.settings.voice_noise_suppression) {
    const subRumbleFilter = audioContext.createBiquadFilter()
    subRumbleFilter.type = 'highpass'
    subRumbleFilter.frequency.value = 85
    subRumbleFilter.Q.value = 0.7

    const notch50 = audioContext.createBiquadFilter()
    notch50.type = 'notch'
    notch50.frequency.value = 50
    notch50.Q.value = 4.0

    const notch60 = audioContext.createBiquadFilter()
    notch60.type = 'notch'
    notch60.frequency.value = 60
    notch60.Q.value = 4.0

    const vocalFormantEQ = audioContext.createBiquadFilter()
    vocalFormantEQ.type = 'peaking'
    vocalFormantEQ.frequency.value = 2600
    vocalFormantEQ.Q.value = 1.1
    vocalFormantEQ.gain.value = 2.5

    const hissCutFilter = audioContext.createBiquadFilter()
    hissCutFilter.type = 'lowpass'
    hissCutFilter.frequency.value = 9500
    hissCutFilter.Q.value = 0.7

    const compressor = audioContext.createDynamicsCompressor()
    compressor.threshold.value = -28
    compressor.knee.value = 8
    compressor.ratio.value = 4
    compressor.attack.value = 0.003
    compressor.release.value = 0.06

    expanderGainNode = audioContext.createGain()
    expanderGainNode.gain.value = 0.001

    source.connect(localAnalyser)
    source.connect(subRumbleFilter)
    subRumbleFilter.connect(notch50)
    notch50.connect(notch60)
    notch60.connect(vocalFormantEQ)
    vocalFormantEQ.connect(hissCutFilter)
    hissCutFilter.connect(compressor)
    compressor.connect(expanderGainNode)
    expanderGainNode.connect(dest)
  } else {
    expanderGainNode = audioContext.createGain()
    expanderGainNode.gain.value = 1

    source.connect(localAnalyser)
    source.connect(expanderGainNode)
    expanderGainNode.connect(dest)
  }

  localStream = dest.stream

  if (voiceState.isMuted && expanderGainNode) {
    expanderGainNode.gain.value = 0
  }
}

export async function setAudioInput(deviceId: string): Promise<void> {
  voiceState.selectedInputId = deviceId
  localStorage.setItem('kip_mic_id', deviceId)

  if (voiceState.isConnected || voiceState.isTestingMic) {
    try {
      await setupDSPChain()
      if (localStream) {
        const newAudioTrack = localStream.getAudioTracks()[0]
        if (voiceState.isConnected && newAudioTrack) {
          Object.values(peers).forEach((peer) => {
            const sender = peer.pc.getSenders().find((s) => s.track?.kind === 'audio')
            if (sender) sender.replaceTrack(newAudioTrack).catch(() => {})
          })
        }
        if (voiceState.isTestingMic && localDummyAudio) {
          localDummyAudio.srcObject = localStream
        }
      }
    } catch {
    }
  }
}

export async function setAudioOutput(deviceId: string): Promise<void> {
  voiceState.selectedOutputId = deviceId
  localStorage.setItem('kip_speaker_id', deviceId)

  if (localDummyAudio && 'setSinkId' in HTMLMediaElement.prototype) {
    (localDummyAudio as unknown as { setSinkId: (id: string) => Promise<void> })
      .setSinkId(deviceId)
      .catch(() => {})
  }

  Object.values(peers).forEach((peer) => {
    if (peer.audioEl && 'setSinkId' in HTMLMediaElement.prototype) {
      (peer.audioEl as unknown as { setSinkId: (id: string) => Promise<void> })
        .setSinkId(deviceId)
        .catch(() => {})
    }
  })
}

export async function startMicTest(): Promise<void> {
  if (voiceState.isTestingMic) return
  if (!voiceState.isConnected) stopMicTest()

  try {
    if (!voiceState.isConnected) {
      await loadAudioDevices()
      await setupDSPChain()
      monitorVoiceActivity()
    }

    voiceState.isTestingMic = true

    localDummyAudio = new Audio()
    localDummyAudio.autoplay = true
    localDummyAudio.volume = 0.8
    localDummyAudio.srcObject = localStream

    if (
      voiceState.selectedOutputId &&
      voiceState.selectedOutputId !== 'default' &&
      'setSinkId' in HTMLMediaElement.prototype
    ) {
      (localDummyAudio as unknown as { setSinkId: (id: string) => Promise<void> })
        .setSinkId(voiceState.selectedOutputId)
        .catch(() => {})
    }

    localDummyAudio.play().catch(() => {})
  } catch {
    showToast(t('Microphone Error'), t('Could not access audio device for testing.'), 'danger')
    stopMicTest()
  }
}

export function stopMicTest(): void {
  voiceState.isTestingMic = false
  voiceState.testMicVolume = 0

  if (localDummyAudio) {
    localDummyAudio.pause()
    localDummyAudio.srcObject = null
    localDummyAudio = null
  }

  if (!voiceState.isConnected) {
    if (vadIntervalId !== null) {
      clearInterval(vadIntervalId)
      vadIntervalId = null
    }
    if (audioContext) {
      audioContext.close().catch(() => {})
      audioContext = null
    }
    if (rawMicStream) {
      rawMicStream.getTracks().forEach((t) => t.stop())
      rawMicStream = null
    }
    localStream = null
    localAnalyser = null
    expanderGainNode = null
  }
}

export async function joinVoiceChannel(channelName: string, host = 'wss://kip-backend.noisyfutlor98.workers.dev/ws'): Promise<void> {
  if (voiceState.isConnected) return
  stopMicTest()

  try {
    await loadAudioDevices()
    await setupDSPChain()
    monitorVoiceActivity()
    initSignaling(channelName, host)

    voiceState.isConnected = true
    voiceState.channelId = channelName
    voiceState.isMuted = false
    voiceState.isDeafened = false

    showToast(t('K.I.P. Connect'), t('Joined voice channel: ') + channelName, 'success')
  } catch (e: unknown) {
    let errorMsg = t('Could not access audio device.')
    if (e instanceof Error && e.name === 'NotAllowedError') {
      errorMsg = 'Microphone access denied by OS permissions.'
    }
    showToast(t('Microphone Error'), errorMsg, 'danger')
  }
}

export function leaveVoiceChannel(): void {
  voiceState.isConnected = false
  voiceState.channelId = ''
  voiceState.localSpeaking = false
  voiceState.participants = []
  state.partyInvite = null

  if (pingInterval) {
    clearInterval(pingInterval)
    pingInterval = null
  }

  if (signalingSocket) {
    signalingSocket.onclose = null
    signalingSocket.close()
    signalingSocket = null
  }

  Object.keys(peers).forEach((peerId) => {
    destroyPeer(peerId)
  })
  peers = {}

  stopMicTest()

  if (vadIntervalId !== null) {
    clearInterval(vadIntervalId)
    vadIntervalId = null
  }
  if (audioContext) {
    audioContext.close().catch(() => {})
    audioContext = null
  }
  if (rawMicStream) {
    rawMicStream.getTracks().forEach((track) => track.stop())
    rawMicStream = null
  }
  localStream = null
  localAnalyser = null
  expanderGainNode = null
}

export function toggleMute(): void {
  if (!localStream) return
  voiceState.isMuted = !voiceState.isMuted

  if (expanderGainNode && audioContext) {
    const targetGain = voiceState.isMuted ? 0 : 1
    expanderGainNode.gain.setTargetAtTime(targetGain, audioContext.currentTime, 0.01)
  }

  if (signalingSocket?.readyState === WebSocket.OPEN) {
    signalingSocket.send(JSON.stringify({ type: 'mute-state', muted: voiceState.isMuted }))
  }
}

export function toggleDeafen(): void {
  voiceState.isDeafened = !voiceState.isDeafened

  Object.values(peers).forEach((peer) => {
    if (peer.audioEl) {
      peer.audioEl.muted = voiceState.isDeafened
    }
  })

  if (voiceState.isDeafened && !voiceState.isMuted) {
    toggleMute()
  }

  if (signalingSocket?.readyState === WebSocket.OPEN) {
    signalingSocket.send(JSON.stringify({ type: 'deafen-state', deafened: voiceState.isDeafened }))
  }
}

function initSignaling(channelId: string, host: string): void {
  const userName = state.settings.has_kip_token
    ? state.settings.kip_username
    : state.settings.ms_name || state.settings.offline_username || 'Guest'

  let targetUrl = host
  if (!targetUrl.startsWith('ws://') && !targetUrl.startsWith('wss://')) {
    targetUrl = (targetUrl.includes('localhost') || targetUrl.includes('127.0.0.1') || targetUrl.includes('192.168.'))
      ? `ws://${targetUrl}`
      : `wss://${targetUrl}`
  }

  const separator = targetUrl.includes('?') ? '&' : '?'
  const wsUrl = `${targetUrl}${separator}channel=${encodeURIComponent(channelId)}&user=${encodeURIComponent(userName)}`

  try {
    signalingSocket = new WebSocket(wsUrl)
  } catch {
    showToast(t('Error'), t('Failed to connect to signaling server.'), 'danger')
    return
  }

  signalingSocket.onopen = () => {
    pingInterval = setInterval(() => {
      if (signalingSocket?.readyState === WebSocket.OPEN) {
        signalingSocket.send(JSON.stringify({ type: 'ping' }))
      }
    }, 12000)
  }

  signalingSocket.onclose = () => {
    if (pingInterval) clearInterval(pingInterval)
    if (voiceState.isConnected) {
      leaveVoiceChannel()
      showToast(t('Disconnected'), t('Signaling server connection lost.'), 'danger')
    }
  }

  signalingSocket.onmessage = async (message: MessageEvent<string>) => {
    try {
      const data = JSON.parse(message.data)
      if (data.type === 'user-joined') {
        createPeerConnection(data.userId, data.userName, Boolean(data.initiator))
      } else if (data.type === 'user-left') {
        destroyPeer(data.userId)
      } else if (data.type === 'offer') {
        if (!peers[data.userId]) {
          createPeerConnection(data.userId, data.userName, false)
        }
        const peer = peers[data.userId]
        if (peer) {
          const pc = peer.pc
          const readyForOffer = !peer.makingOffer && (pc.signalingState === 'stable' || peer.isSettingRemoteAnswerPending)
          const offerCollision = !readyForOffer
          peer.ignoreOffer = !peer.polite && offerCollision
          if (peer.ignoreOffer) {
            return
          }
          try {
            if (offerCollision) {
              await pc.setLocalDescription({ type: 'rollback' })
            }
            await pc.setRemoteDescription(new RTCSessionDescription(data.sdp || data.offer))
            await pc.setLocalDescription()
            if (signalingSocket?.readyState === WebSocket.OPEN) {
              signalingSocket.send(
                JSON.stringify({
                  type: 'answer',
                  sdp: pc.localDescription,
                  target: data.userId
                })
              )
            }
            for (const cand of peer.pendingCandidates) {
              await pc.addIceCandidate(new RTCIceCandidate(cand)).catch(() => {})
            }
            peer.pendingCandidates = []
          } catch {
          }
        }
      } else if (data.type === 'answer') {
        const peer = peers[data.userId]
        if (peer && peer.pc) {
          try {
            await peer.pc.setRemoteDescription(new RTCSessionDescription(data.sdp || data.answer))
            for (const cand of peer.pendingCandidates) {
              await peer.pc.addIceCandidate(new RTCIceCandidate(cand)).catch(() => {})
            }
            peer.pendingCandidates = []
          } catch {
          }
        }
      } else if (data.type === 'ice-candidate') {
        const peer = peers[data.userId]
        if (peer && peer.pc) {
          const candidateInit = data.candidate
          if (candidateInit) {
            if (peer.pc.remoteDescription && peer.pc.remoteDescription.type) {
              await peer.pc.addIceCandidate(new RTCIceCandidate(candidateInit)).catch(() => {})
            } else {
              peer.pendingCandidates.push(candidateInit)
            }
          }
        }
      } else if (data.type === 'mute-state') {
        const p = voiceState.participants.find((part) => part.id === data.userId)
        if (p) p.muted = data.muted
      } else if (data.type === 'deafen-state') {
        const p = voiceState.participants.find((part) => part.id === data.userId)
        if (p) p.deafened = data.deafened
      } else if (data.type === 'party-invite') {
        state.partyInvite = {
          senderId: data.userId,
          senderName: data.userName,
          mods: Array.isArray(data.mods) ? data.mods : [],
          tunnelUrl: data.tunnelUrl || ''
        }
      } else if (data.type === 'party-accept') {
        showToast(t('K.I.P. Party'), `${data.userName} accepted your invite!`, 'success')
      } else if (data.type === 'party-decline') {
        showToast(t('K.I.P. Party'), `${data.userName} declined your invite.`, 'danger')
      }
    } catch {
    }
  }
}

function createPeerConnection(peerId: string, peerName: string, isInitiator: boolean): void {
  const pc = new RTCPeerConnection(RTC_CONFIG)
  const polite = !isInitiator

  const peer: WebRTCPeer = {
    pc,
    analyser: null,
    audioCtx: null,
    audioEl: null,
    name: peerName,
    makingOffer: false,
    ignoreOffer: false,
    isSettingRemoteAnswerPending: false,
    pendingCandidates: [],
    lastSpokeTime: 0,
    polite
  }

  peers[peerId] = peer

  voiceState.participants.push({
    id: peerId,
    name: peerName,
    speaking: false,
    muted: false,
    deafened: false
  })

  if (localStream) {
    localStream.getTracks().forEach((track) => {
      if (localStream) pc.addTrack(track, localStream)
    })
  }

  pc.onicecandidate = (event: RTCPeerConnectionIceEvent) => {
    if (event.candidate && signalingSocket?.readyState === WebSocket.OPEN) {
      signalingSocket.send(
        JSON.stringify({
          type: 'ice-candidate',
          candidate: event.candidate.toJSON(),
          target: peerId
        })
      )
    }
  }

  pc.onnegotiationneeded = async () => {
    try {
      peer.makingOffer = true
      await pc.setLocalDescription()
      if (signalingSocket?.readyState === WebSocket.OPEN) {
        signalingSocket.send(
          JSON.stringify({
            type: 'offer',
            sdp: pc.localDescription,
            target: peerId
          })
        )
      }
    } catch {
    } finally {
      peer.makingOffer = false
    }
  }

  pc.oniceconnectionstatechange = () => {
    if (
      pc.iceConnectionState === 'failed' ||
      pc.iceConnectionState === 'closed' ||
      pc.iceConnectionState === 'disconnected'
    ) {
      if (pc.iceConnectionState === 'failed') {
        pc.restartIce()
      }
    }
  }

  pc.ontrack = (event: RTCTrackEvent) => {
    const activePeer = peers[peerId]
    if (!activePeer) return

    const stream = event.streams[0] || new MediaStream([event.track])
    const audioEl = new Audio()
    audioEl.autoplay = true
    audioEl.muted = voiceState.isDeafened
    audioEl.srcObject = stream

    if (
      voiceState.selectedOutputId &&
      voiceState.selectedOutputId !== 'default' &&
      'setSinkId' in HTMLMediaElement.prototype
    ) {
      (audioEl as unknown as { setSinkId: (id: string) => Promise<void> })
        .setSinkId(voiceState.selectedOutputId)
        .catch(() => {})
    }

    const startAudioPlay = () => {
      audioEl.play().catch(() => {
        const unblockHandler = () => {
          audioEl.play().catch(() => {})
          window.removeEventListener('click', unblockHandler)
          window.removeEventListener('keydown', unblockHandler)
        }
        window.addEventListener('click', unblockHandler)
        window.addEventListener('keydown', unblockHandler)
      })
    }
    startAudioPlay()

    const AudioCtxClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext
    const peerAudioCtx = new AudioCtxClass()
    peerAudioCtx.resume().catch(() => {})

    const source = peerAudioCtx.createMediaStreamSource(stream)
    const analyser = peerAudioCtx.createAnalyser()
    analyser.fftSize = 512
    analyser.smoothingTimeConstant = 0.4

    const dummyDest = peerAudioCtx.createMediaStreamDestination()
    source.connect(analyser)
    analyser.connect(dummyDest)

    activePeer.audioEl = audioEl
    activePeer.analyser = analyser
    activePeer.audioCtx = peerAudioCtx
  }
}

function destroyPeer(peerId: string): void {
  const peer = peers[peerId]
  if (peer) {
    if (peer.pc) peer.pc.close()
    if (peer.audioEl) {
      peer.audioEl.pause()
      peer.audioEl.srcObject = null
    }
    if (peer.analyser) peer.analyser.disconnect()
    if (peer.audioCtx) peer.audioCtx.close().catch(() => {})
    delete peers[peerId]
  }
  voiceState.participants = voiceState.participants.filter((p) => p.id !== peerId)
}

function monitorVoiceActivity(): void {
  if (vadIntervalId !== null) {
    clearInterval(vadIntervalId)
    vadIntervalId = null
  }

  const HYSTERESIS_MS = 240

  vadIntervalId = setInterval(() => {
    if (!voiceState.isConnected && !voiceState.isTestingMic) {
      if (vadIntervalId !== null) {
        clearInterval(vadIntervalId)
        vadIntervalId = null
      }
      return
    }

    const now = Date.now()

    if (localAnalyser) {
      const data = new Uint8Array(localAnalyser.frequencyBinCount)
      localAnalyser.getByteFrequencyData(data)

      let formantEnergy = 0
      for (let i = 3; i <= 36; i++) {
        formantEnergy += data[i]
      }
      formantEnergy /= 34

      let outOfBandNoise = 0
      for (let i = 45; i < 90; i++) {
        outOfBandNoise += data[i]
      }
      outOfBandNoise /= 45

      if (formantEnergy < adaptiveNoiseFloor) {
        adaptiveNoiseFloor = adaptiveNoiseFloor * 0.9 + formantEnergy * 0.1
      } else {
        adaptiveNoiseFloor += 0.03
      }
      if (adaptiveNoiseFloor < 4) adaptiveNoiseFloor = 4

      const isHumanVoice = (formantEnergy > adaptiveNoiseFloor * 1.45 + 5) && (formantEnergy >= outOfBandNoise * 0.85) && !voiceState.isMuted

      if (isHumanVoice) {
        localLastSpokeTime = now
        if (expanderGainNode && audioContext && state.settings.voice_noise_suppression) {
          expanderGainNode.gain.setTargetAtTime(1.0, audioContext.currentTime, 0.008)
        }
      } else {
        if (now - localLastSpokeTime > HYSTERESIS_MS) {
          if (expanderGainNode && audioContext && state.settings.voice_noise_suppression) {
            expanderGainNode.gain.setTargetAtTime(0.001, audioContext.currentTime, 0.08)
          }
        }
      }

      voiceState.localSpeaking = now - localLastSpokeTime < HYSTERESIS_MS
      if (voiceState.isTestingMic) {
        voiceState.testMicVolume = Math.min(Math.round((formantEnergy / 255) * 100), 100)
      }
    }

    voiceState.participants.forEach((p) => {
      const peer = peers[p.id]
      if (peer?.analyser) {
        const data = new Uint8Array(peer.analyser.frequencyBinCount)
        peer.analyser.getByteFrequencyData(data)

        let formantEnergy = 0
        for (let i = 3; i <= 36; i++) {
          formantEnergy += data[i]
        }
        formantEnergy /= 34

        const isSpeaking = formantEnergy > 12 && !p.muted
        if (isSpeaking) {
          peer.lastSpokeTime = now
        }
        p.speaking = now - (peer.lastSpokeTime || 0) < HYSTERESIS_MS
      }
    })
  }, 35)
}

export async function inviteToParty(targetUserId: string): Promise<void> {
  if (signalingSocket?.readyState !== WebSocket.OPEN) return
  try {
    const data = await bridge.partyInvitePrepare()
    signalingSocket.send(
      JSON.stringify({
        type: 'party-invite',
        target: targetUserId,
        mods: data.mods,
        tunnelUrl: data.tunnel_url
      })
    )
    showToast(t('K.I.P. Party'), t('Invite sent!'), 'success')
  } catch {
    showToast(t('Error'), t('Failed to prepare party invite.'), 'danger')
  }
}

export async function acceptPartyInvite(): Promise<void> {
  const invite = state.partyInvite
  if (!invite) return
  state.partyInvite = null

  showToast(t('K.I.P. Party'), t('Accepting invite and syncing mods...'), 'info')

  let host = 'wss://kip-backend.noisyfutlor98.workers.dev/ws'
  if (invite.tunnelUrl) {
    const match = invite.tunnelUrl.match(/(?:tcp:\/\/|https:\/\/|wss:\/\/)([a-zA-Z0-9.\-]+(?::\d+)?)/)
    if (match) host = match[1]
  }

  if (signalingSocket?.readyState === WebSocket.OPEN) {
    signalingSocket.send(
      JSON.stringify({
        type: 'party-accept',
        target: invite.senderId
      })
    )
  }

  if (invite.tunnelUrl) {
    try {
      await navigator.clipboard.writeText(invite.tunnelUrl)
      showToast(t('K.I.P. Party'), t('Server IP copied to clipboard!'), 'success')
    } catch {
    }
  }

  setTimeout(() => {
    joinVoiceChannel('kip-party', host)
  }, 1000)
}

export function declinePartyInvite(): void {
  const invite = state.partyInvite
  if (!invite) return
  state.partyInvite = null
  if (signalingSocket?.readyState === WebSocket.OPEN) {
    signalingSocket.send(
      JSON.stringify({
        type: 'party-decline',
        target: invite.senderId
      })
    )
  }
}

interface GamepadNavigationState {
  active: boolean
  focusedEl: HTMLElement | null
  lastInput: number
  raf: number | null
}

const gamepadState: GamepadNavigationState = {
  active: false,
  focusedEl: null,
  lastInput: 0,
  raf: null
}

export function initGamepadMode(): void {
  window.addEventListener('gamepadconnected', () => {
    gamepadState.active = true
    if (!gamepadState.raf) gamepadLoop()
  })

  window.addEventListener('gamepaddisconnected', () => {
    const gps = navigator.getGamepads ? navigator.getGamepads() : []
    let anyActive = false
    for (let i = 0; i < gps.length; i++) {
      if (gps[i]) anyActive = true
    }
    if (!anyActive) {
      gamepadState.active = false
      if (gamepadState.focusedEl) {
        gamepadState.focusedEl.classList.remove('gamepad-focus')
        gamepadState.focusedEl = null
      }
    }
  })
}

function switchView(dir: number): void {
  const views = state.navItems.map((n) => n.id)
  let currIdx = views.indexOf(state.currentView)
  currIdx += dir
  if (currIdx < 0) currIdx = views.length - 1
  if (currIdx >= views.length) currIdx = 0
  state.currentView = views[currIdx]
}

function getInteractables(): HTMLElement[] {
  let root: HTMLElement = document.body
  const activeModals = Array.from(
    document.querySelectorAll<HTMLElement>(
      '.fixed.z-\\[200\\], .fixed.z-\\[300\\], .fixed.z-\\[10000\\], .fixed.z-\\[99999\\]'
    )
  ).filter((el) => {
    const style = window.getComputedStyle(el)
    return (
      style.display !== 'none' &&
      style.opacity !== '0' &&
      style.visibility !== 'hidden' &&
      style.pointerEvents !== 'none' &&
      el.getBoundingClientRect().width > 0
    )
  })

  if (activeModals.length > 0) {
    root = activeModals.sort(
      (a, b) => Number(window.getComputedStyle(b).zIndex) - Number(window.getComputedStyle(a).zIndex)
    )[0]
  } else if (state.isNexusOpen) {
    root = document.querySelector<HTMLElement>('.z-\\[100\\]') || document.body
  }

  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'button, input, select, textarea, a, .cursor-pointer, .kip-card-hover'
    )
  ).filter((el) => {
    const rect = el.getBoundingClientRect()
    const style = window.getComputedStyle(el)
    if (
      style.visibility === 'hidden' ||
      style.opacity === '0' ||
      style.display === 'none' ||
      style.pointerEvents === 'none' ||
      (el as HTMLButtonElement).disabled
    ) {
      return false
    }
    if (state.isBigPicture && el.closest('.titlebar')) return false
    return rect.width > 0 && rect.height > 0
  })
}

function gamepadLoop(): void {
  if (!gamepadState.active) {
    gamepadState.raf = null
    return
  }

  let gp: Gamepad | null = null
  const gps = navigator.getGamepads ? navigator.getGamepads() : []
  for (let i = 0; i < gps.length; i++) {
    if (gps[i]) {
      gp = gps[i]
      break
    }
  }

  if (gp) {
    const now = Date.now()
    let dx = 0
    let dy = 0
    if (gp.buttons[12]?.pressed || gp.axes[1] < -0.5) dy = -1
    if (gp.buttons[13]?.pressed || gp.axes[1] > 0.5) dy = 1
    if (gp.buttons[14]?.pressed || gp.axes[0] < -0.5) dx = -1
    if (gp.buttons[15]?.pressed || gp.axes[0] > 0.5) dx = 1

    const btnL1 = gp.buttons[4]?.pressed
    const btnR1 = gp.buttons[5]?.pressed
    const btnA = gp.buttons[0]?.pressed
    const btnB = gp.buttons[1]?.pressed

    const hasAction = dx !== 0 || dy !== 0 || btnL1 || btnR1 || btnA || btnB

    if (hasAction && now - gamepadState.lastInput > 150) {
      if (dx !== 0 || dy !== 0) {
        moveGamepadFocus(dx, dy)
        gamepadState.lastInput = now
      } else if (btnL1) {
        switchView(-1)
        gamepadState.lastInput = now + 200
      } else if (btnR1) {
        switchView(1)
        gamepadState.lastInput = now + 200
      } else if (btnA) {
        if (gamepadState.focusedEl && document.body.contains(gamepadState.focusedEl)) {
          gamepadState.focusedEl.click()
          gamepadState.focusedEl.classList.add('scale-95')
          setTimeout(() => {
            if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('scale-95')
          }, 100)
          gamepadState.lastInput = now + 200
        }
      } else if (btnB) {
        const interactables = getInteractables()
        const closeBtns = interactables.filter(
          (b) =>
            b.innerHTML.includes('lucide-x') ||
            b.textContent?.includes('Close') ||
            b.textContent?.includes('Cancel') ||
            b.textContent?.includes('Decline')
        )
        const visibleCloseBtn = closeBtns.reverse()[0]
        if (visibleCloseBtn) {
          visibleCloseBtn.click()
        } else if (state.isNexusOpen) {
          state.isNexusOpen = false
        } else if (state.isBigPicture) {
          toggleBigPicture()
        }
        gamepadState.lastInput = now + 200
      }
    }
  }

  gamepadState.raf = requestAnimationFrame(gamepadLoop)
}

function moveGamepadFocus(dx: number, dy: number): void {
  const interactables = getInteractables()
  if (interactables.length === 0) return

  if (
    !gamepadState.focusedEl ||
    !document.body.contains(gamepadState.focusedEl) ||
    !interactables.includes(gamepadState.focusedEl)
  ) {
    setGamepadFocus(interactables[0])
    return
  }

  const currentRect = gamepadState.focusedEl.getBoundingClientRect()
  const currentCenterX = currentRect.left + currentRect.width / 2
  const currentCenterY = currentRect.top + currentRect.height / 2

  let bestMatch: HTMLElement | null = null
  let minDistance = Infinity

  for (const el of interactables) {
    if (el === gamepadState.focusedEl) continue

    const rect = el.getBoundingClientRect()
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2

    const rawX = centerX - currentCenterX
    const rawY = centerY - currentCenterY
    const diffX = Math.abs(rawX)
    const diffY = Math.abs(rawY)

    let isValidDirection = false
    if (dx === 1 && rawX > 0 && diffX >= diffY) isValidDirection = true
    if (dx === -1 && rawX < 0 && diffX >= diffY) isValidDirection = true
    if (dy === 1 && rawY > 0 && diffY >= diffX) isValidDirection = true
    if (dy === -1 && rawY < 0 && diffY >= diffX) isValidDirection = true

    if (isValidDirection) {
      const distance = Math.sqrt(rawX * rawX + rawY * rawY)
      if (distance < minDistance) {
        minDistance = distance
        bestMatch = el
      }
    }
  }

  if (bestMatch) setGamepadFocus(bestMatch)
}

function setGamepadFocus(el: HTMLElement): void {
  if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('gamepad-focus')
  gamepadState.focusedEl = el
  el.classList.add('gamepad-focus')
  el.focus({ preventScroll: true })
  el.scrollIntoView({ behavior: 'auto', block: 'center', inline: 'center' })
}

export { setupTauriListeners }