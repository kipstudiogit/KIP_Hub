import { reactive } from 'vue'
import { bridge } from '../bridge'
import type { VoiceState, WebRTCPeer } from '../types/voice'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'

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

function getValidDeviceId(savedId: string, devices: MediaDeviceInfo[]): string {
  if (!savedId || savedId === 'default') return 'default'
  const exists = devices.find((d) => d.deviceId === savedId)
  return exists ? savedId : 'default'
}

export async function loadAudioDevices(): Promise<void> {
  if (!navigator.mediaDevices?.enumerateDevices) return

  try {
    let tempStream: MediaStream | null = null
    try {
      tempStream = await navigator.mediaDevices.getUserMedia({ audio: true })
    } catch {
      tempStream = null
    }

    const devices = await navigator.mediaDevices.enumerateDevices()
    voiceState.inputDevices = devices.filter((d) => d.kind === 'audioinput')
    voiceState.outputDevices = devices.filter((d) => d.kind === 'audiooutput')

    if (tempStream) {
      tempStream.getTracks().forEach((trk) => trk.stop())
    }

    if (!isDeviceListenerAdded) {
      navigator.mediaDevices.addEventListener('devicechange', loadAudioDevices)
      isDeviceListenerAdded = true
    }
  } catch {
    voiceState.inputDevices = []
    voiceState.outputDevices = []
  }
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
    rawMicStream.getTracks().forEach((trk) => trk.stop())
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
      showToast(t('Error'), t('Failed to switch audio input device.'), 'danger')
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
      rawMicStream.getTracks().forEach((trk) => trk.stop())
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
            showToast(t('Audio Error'), t('WebRTC offer resolution failed.'), 'danger')
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
            showToast(t('Audio Error'), t('WebRTC answer resolution failed.'), 'danger')
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
      showToast(t('Signaling Error'), t('Failed to process network frame.'), 'danger')
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
      showToast(t('Audio Error'), t('Negotiation exchange failed.'), 'danger')
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
      showToast(t('Warning'), t('Could not copy server IP automatically.'), 'danger')
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