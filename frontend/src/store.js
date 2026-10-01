import { reactive, ref } from 'vue'
import { bridge, setupTauriListeners } from './bridge.js'

export const api = ref(bridge)

export const state = reactive({
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
        instances: []
    },
    friends: [],
    newsText: '',
    consoleHtml: 'Awaiting data stream...',
    _consoleBuffer: []
})

export const voiceState = reactive({
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

export const toasts = ref([])
let toastIdCounter = 0

let rawMicStream = null
let localStream = null
let localDummyAudio = null
let audioContext = null
let localAnalyser = null
let gateGainNode = null
let noiseFloor = 15
let vadAnimationId = null

let signalingSocket = null
let pingInterval = null
let peers = {}

let localLastSpokeTime = 0
let isDeviceListenerAdded = false

const RTC_CONFIG = {
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' }
    ]
}

export function showToast(title, message, type = "info") {
    const id = toastIdCounter++
    let icon = "info"
    if (type === "success") icon = "check-circle"
    if (type === "danger") icon = "x-circle"

    toasts.value.push({ id, title, message, type, icon })
    if (toasts.value.length > 5) toasts.value.shift()

    setTimeout(() => {
        toasts.value = toasts.value.filter(t => t.id !== id)
    }, 4000)
}

export function t(key) {
    if (!key) return ''
    const lang = state.settings.lang
    if (!lang || !state.translations[lang]) return key
    return state.translations[lang][key] || key
}

export function sanitizeHTML(str) {
    const temp = document.createElement('div')
    temp.textContent = str
    return temp.innerHTML
}

export function getAvatarUrl(name) {
    if (!name) return ''
    return `https://mc-heads.net/avatar/${encodeURIComponent(name)}/100`
}

export async function loadTranslations() {
    const lang = state.settings.lang
    if (!lang) return
    if (Object.keys(state.translations[lang] || {}).length === 0) {
        try {
            const dict = await bridge.getTranslations(lang)
            if (dict && Object.keys(dict).length > 0) {
                state.translations[lang] = dict
            }
        } catch (e) {}
    }
}

export async function loadSettings() {
    try {
        const s = await bridge.getSettings()
        if (s) {
            state.settings.mc_dir = s.mc_dir
            state.settings.auto_backup = s.auto_backup
            state.settings.rpc = s.rpc
            state.settings.ai_provider = s.ai_provider || 'google'
            state.settings.ai_api_key = s.ai_api_key || ''
            state.settings.openai_api_key = s.openai_api_key || ''
            state.settings.anthropic_api_key = s.anthropic_api_key || ''
            state.settings.ollama_url = s.ollama_url || 'http://localhost:11434'
            state.settings.cf_api_key = s.cf_api_key || ''
            state.settings.lang = s.lang
            state.settings.autostart = s.autostart
            state.settings.safe_mode = s.safe_mode
            state.settings.low_graphics = s.low_graphics || false
            state.settings.close_on_launch = s.close_on_launch || false
            state.settings.ram_allocation = s.ram_allocation || 0
            state.settings.shield_auto_scan = s.shield_auto_scan !== false
            state.settings.voice_noise_suppression = s.voice_noise_suppression !== false
            state.settings.eula_accepted = s.eula_accepted || false
            state.settings.instances = s.instances || []
            state.settings.offline_username = s.offline_username || 'Player'
            state.settings.game_resolution = s.game_resolution || '854x480'
            state.settings.game_fullscreen = !!s.game_fullscreen
            state.settings.custom_java_path = s.custom_java_path || ''
            state.settings.custom_jvm_args = s.custom_jvm_args || ''
        }

        const prof = await bridge.getMsProfile()
        if (prof && prof.success) {
            state.settings.has_ms_token = true
            state.settings.ms_name = prof.name
        } else {
            state.settings.has_ms_token = false
            state.settings.ms_name = ''
        }

        const kipProf = await bridge.getKipProfile()
        if (kipProf && kipProf.success) {
            state.settings.has_kip_token = true
            state.settings.kip_username = kipProf.username
        } else {
            state.settings.has_kip_token = false
            state.settings.kip_username = ''
        }
    } catch (e) {}
}

export async function loadDashboardStats() {
    try {
        const s = await bridge.getDashboardStats()
        if (s) {
            state.stats.size = s.size
            state.stats.saves = s.saves
            state.stats.playtime = s.playtime
            state.stats.java = s.java
        }
    } catch (e) {}
}

export async function saveSetting(key, value) {
    try {
        state.settings[key] = value
        const result = await bridge.saveSetting(key, value)
        if (key === 'lang') {
            await loadTranslations()
        }
        return result
    } catch (e) {
        return false
    }
}

export async function acceptEula() {
    state.settings.eula_accepted = true
    await saveSetting('eula_accepted', true)
}

export function windowMinimize() {
    bridge.windowMinimize()
}

export function windowMaximize() {
    bridge.windowMaximize()
}

export function windowClose() {
    bridge.windowClose()
}

export function toggleBigPicture() {
    state.isBigPicture = !state.isBigPicture
    bridge.toggleBigPicture().catch(() => {})
}

export function toggleMiniMode() {
    state.isMiniMode = !state.isMiniMode
    bridge.setMiniMode(state.isMiniMode).catch(() => {})
}

export async function loadAudioDevices() {
    if (!navigator.mediaDevices?.enumerateDevices) return

    try {
        let tempStream = null
        try {
            tempStream = await navigator.mediaDevices.getUserMedia({ audio: true })
        } catch (e) {}

        const devices = await navigator.mediaDevices.enumerateDevices()
        voiceState.inputDevices = devices.filter(d => d.kind === 'audioinput')
        voiceState.outputDevices = devices.filter(d => d.kind === 'audiooutput')

        if (tempStream) {
            tempStream.getTracks().forEach(t => t.stop())
        }

        if (!isDeviceListenerAdded) {
            navigator.mediaDevices.addEventListener('devicechange', loadAudioDevices)
            isDeviceListenerAdded = true
        }
    } catch (e) {}
}

function getValidDeviceId(savedId, devices) {
    if (!savedId || savedId === 'default') return 'default'
    const exists = devices.find(d => d.deviceId === savedId)
    return exists ? savedId : 'default'
}

export async function toggleVoiceSettings() {
    voiceState.showSettings = !voiceState.showSettings
    if (voiceState.showSettings) {
        await loadAudioDevices()
    } else {
        stopMicTest()
    }
}

async function setupDSPChain() {
    if (rawMicStream) {
        rawMicStream.getTracks().forEach(t => t.stop())
    }

    const validDeviceId = getValidDeviceId(voiceState.selectedInputId, voiceState.inputDevices)
    const constraints = {
        audio: {
            echoCancellation: true,
            noiseSuppression: true,
            autoGainControl: true,
            sampleRate: 48000,
            channelCount: 1,
            googEchoCancellation: true,
            googAutoGainControl: true,
            googNoiseSuppression: true,
            googHighpassFilter: true,
            googAudioMirroring: false,
            googNoiseReduction: true
        },
        video: false
    }

    if (validDeviceId !== 'default') {
        constraints.audio.deviceId = { exact: validDeviceId }
    }

    rawMicStream = await navigator.mediaDevices.getUserMedia(constraints)

    if (audioContext) {
        audioContext.close()
    }

    audioContext = new (window.AudioContext || window.webkitAudioContext)()
    await audioContext.resume()

    const source = audioContext.createMediaStreamSource(rawMicStream)
    const dest = audioContext.createMediaStreamDestination()

    localAnalyser = audioContext.createAnalyser()
    localAnalyser.fftSize = 512
    localAnalyser.smoothingTimeConstant = 0.5
    source.connect(localAnalyser)

    if (state.settings.voice_noise_suppression !== false) {
        const hpFilter = audioContext.createBiquadFilter()
        hpFilter.type = 'highpass'
        hpFilter.frequency.value = 85

        const lpFilter = audioContext.createBiquadFilter()
        lpFilter.type = 'lowpass'
        lpFilter.frequency.value = 7500

        const eq = audioContext.createBiquadFilter()
        eq.type = 'peaking'
        eq.frequency.value = 3000
        eq.Q.value = 1.0
        eq.gain.value = 2.5

        const compressor = audioContext.createDynamicsCompressor()
        compressor.threshold.value = -35
        compressor.knee.value = 15
        compressor.ratio.value = 8
        compressor.attack.value = 0.005
        compressor.release.value = 0.1

        gateGainNode = audioContext.createGain()
        gateGainNode.gain.value = 0

        source.connect(hpFilter)
        hpFilter.connect(lpFilter)
        lpFilter.connect(eq)
        eq.connect(compressor)
        compressor.connect(gateGainNode)
        gateGainNode.connect(dest)
    } else {
        gateGainNode = audioContext.createGain()
        gateGainNode.gain.value = 1
        source.connect(gateGainNode)
        gateGainNode.connect(dest)
    }

    localStream = dest.stream

    if (voiceState.isMuted && gateGainNode) {
        gateGainNode.gain.value = 0
    }
}

export async function setAudioInput(deviceId) {
    voiceState.selectedInputId = deviceId
    localStorage.setItem('kip_mic_id', deviceId)

    if (voiceState.isConnected || voiceState.isTestingMic) {
        try {
            await setupDSPChain()
            const newAudioTrack = localStream.getAudioTracks()[0]

            if (voiceState.isConnected) {
                Object.values(peers).forEach(peer => {
                    const sender = peer.pc.getSenders().find(s => s.track?.kind === 'audio')
                    if (sender) sender.replaceTrack(newAudioTrack).catch(() => {})
                })
            }

            if (voiceState.isTestingMic && localDummyAudio) {
                localDummyAudio.srcObject = localStream
            }
        } catch (e) {}
    }
}

export async function setAudioOutput(deviceId) {
    voiceState.selectedOutputId = deviceId
    localStorage.setItem('kip_speaker_id', deviceId)

    if (voiceState.isTestingMic && localDummyAudio && typeof localDummyAudio.setSinkId === 'function') {
        localDummyAudio.setSinkId(deviceId).catch(() => {})
    }

    Object.values(peers).forEach(peer => {
        if (peer.audioEl && typeof peer.audioEl.setSinkId === 'function') {
            peer.audioEl.setSinkId(deviceId).catch(() => {})
        }
    })
}

export async function startMicTest() {
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
        localDummyAudio.srcObject = localStream

        if (voiceState.selectedOutputId && voiceState.selectedOutputId !== 'default' && typeof localDummyAudio.setSinkId === 'function') {
            localDummyAudio.setSinkId(voiceState.selectedOutputId).catch(() => {})
        }

        localDummyAudio.play().catch(() => {})
    } catch (e) {
        showToast(t("Microphone Error"), t("Could not access audio device for testing."), "danger")
        stopMicTest()
    }
}

export function stopMicTest() {
    voiceState.isTestingMic = false
    voiceState.testMicVolume = 0

    if (localDummyAudio) {
        localDummyAudio.pause()
        localDummyAudio.srcObject = null
        localDummyAudio = null
    }

    if (!voiceState.isConnected) {
        if (vadAnimationId) {
            cancelAnimationFrame(vadAnimationId)
            vadAnimationId = null
        }
        if (audioContext) {
            audioContext.close()
            audioContext = null
        }
        if (rawMicStream) {
            rawMicStream.getTracks().forEach(t => t.stop())
            rawMicStream = null
        }
        localStream = null
        localAnalyser = null
        gateGainNode = null
    }
}

export async function joinVoiceChannel(channelName, host = '127.0.0.1:8765') {
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

        showToast(t("K.I.P. Connect"), t("Joined voice channel: ") + channelName, "success")
    } catch (e) {
        let errorMsg = t("Could not access audio device.")
        if (e.name === 'NotAllowedError') errorMsg = "Microphone access denied by OS."
        showToast(t("Microphone Error"), errorMsg, "danger")
    }
}

export function leaveVoiceChannel() {
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

    Object.keys(peers).forEach(peerId => {
        destroyPeer(peerId)
    })
    peers = {}

    stopMicTest()

    if (vadAnimationId) {
        cancelAnimationFrame(vadAnimationId)
        vadAnimationId = null
    }
    if (audioContext) {
        audioContext.close()
        audioContext = null
    }
    if (rawMicStream) {
        rawMicStream.getTracks().forEach(track => track.stop())
        rawMicStream = null
    }
    localStream = null
    localAnalyser = null
    gateGainNode = null
}

export function toggleMute() {
    if (!localStream) return
    voiceState.isMuted = !voiceState.isMuted

    if (gateGainNode && audioContext) {
        const targetGain = voiceState.isMuted ? 0 : 1
        gateGainNode.gain.setTargetAtTime(targetGain, audioContext.currentTime, 0.01)
    }

    if (signalingSocket?.readyState === WebSocket.OPEN) {
        signalingSocket.send(JSON.stringify({ type: 'mute-state', muted: voiceState.isMuted }))
    }
}

export function toggleDeafen() {
    voiceState.isDeafened = !voiceState.isDeafened

    Object.values(peers).forEach(peer => {
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

function initSignaling(channelId, host = '127.0.0.1:8765') {
    const userName = state.settings.has_kip_token ? state.settings.kip_username : (state.settings.ms_name || state.settings.offline_username || 'Guest')
    const wsProtocol = host.includes('pinggy.io') ? 'wss://' : 'ws://'
    const wsUrl = `${wsProtocol}${host}?channel=${channelId}&user=${encodeURIComponent(userName)}`

    try {
        signalingSocket = new WebSocket(wsUrl)
    } catch (e) {
        showToast(t("Error"), t("Failed to connect to signaling server."), "danger")
        return
    }

    signalingSocket.onopen = () => {
        pingInterval = setInterval(() => {
            if (signalingSocket?.readyState === WebSocket.OPEN) {
                signalingSocket.send(JSON.stringify({ type: 'ping' }))
            }
        }, 15000)
    }

    signalingSocket.onclose = () => {
        if (pingInterval) clearInterval(pingInterval)
        if (voiceState.isConnected) {
            leaveVoiceChannel()
            showToast(t("Disconnected"), t("Signaling server connection lost."), "danger")
        }
    }

    signalingSocket.onmessage = async (message) => {
        try {
            const data = JSON.parse(message.data)

            if (data.type === 'user-joined') {
                createPeerConnection(data.userId, data.userName, !!data.initiator)
            } else if (data.type === 'user-left') {
                destroyPeer(data.userId)
            } else if (data.type === 'offer') {
                if (!peers[data.userId]) {
                    createPeerConnection(data.userId, data.userName, false)
                }
                const peer = peers[data.userId]
                await peer.pc.setRemoteDescription(new RTCSessionDescription(data.offer))
                const answer = await peer.pc.createAnswer()
                await peer.pc.setLocalDescription(answer)
                signalingSocket.send(JSON.stringify({ type: 'answer', answer, target: data.userId }))

                peer.pendingCandidates.forEach(c => peer.pc.addIceCandidate(c).catch(() => {}))
                peer.pendingCandidates = []
            } else if (data.type === 'answer') {
                const peer = peers[data.userId]
                if (peer?.pc) {
                    await peer.pc.setRemoteDescription(new RTCSessionDescription(data.answer))
                    peer.pendingCandidates.forEach(c => peer.pc.addIceCandidate(c).catch(() => {}))
                    peer.pendingCandidates = []
                }
            } else if (data.type === 'ice-candidate') {
                const peer = peers[data.userId]
                if (peer?.pc) {
                    if (peer.pc.remoteDescription?.type) {
                        peer.pc.addIceCandidate(new RTCIceCandidate(data.candidate)).catch(() => {})
                    } else {
                        peer.pendingCandidates.push(new RTCIceCandidate(data.candidate))
                    }
                }
            } else if (data.type === 'mute-state') {
                const p = voiceState.participants.find(part => part.id === data.userId)
                if (p) p.muted = data.muted
            } else if (data.type === 'deafen-state') {
                const p = voiceState.participants.find(part => part.id === data.userId)
                if (p) p.deafened = data.deafened
            } else if (data.type === 'party-invite') {
                state.partyInvite = {
                    senderId: data.userId,
                    senderName: data.userName,
                    mods: data.mods || [],
                    tunnelUrl: data.tunnelUrl || ''
                }
            } else if (data.type === 'party-accept') {
                showToast(t("K.I.P. Party"), `${data.userName} accepted your invite!`, "success")
            } else if (data.type === 'party-decline') {
                showToast(t("K.I.P. Party"), `${data.userName} declined your invite.`, "danger")
            }
        } catch (e) {}
    }
}

function createPeerConnection(peerId, peerName, isInitiator) {
    const pc = new RTCPeerConnection(RTC_CONFIG)

    peers[peerId] = {
        pc,
        analyser: null,
        audioCtx: null,
        audioEl: null,
        name: peerName,
        pendingCandidates: [],
        lastSpokeTime: 0
    }

    voiceState.participants.push({
        id: peerId,
        name: peerName,
        speaking: false,
        muted: false,
        deafened: false
    })

    if (localStream) {
        localStream.getTracks().forEach(track => {
            pc.addTrack(track, localStream)
        })
    }

    pc.onicecandidate = (event) => {
        if (event.candidate && signalingSocket?.readyState === WebSocket.OPEN) {
            signalingSocket.send(JSON.stringify({
                type: 'ice-candidate',
                candidate: event.candidate,
                target: peerId
            }))
        }
    }

    pc.oniceconnectionstatechange = () => {
        if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'closed' || pc.iceConnectionState === 'disconnected') {
            destroyPeer(peerId)
        }
    }

    pc.ontrack = (event) => {
        if (!peers[peerId]) return

        const stream = event.streams?.[0] || new MediaStream([event.track])

        const audioEl = new Audio()
        audioEl.autoplay = true
        audioEl.muted = voiceState.isDeafened
        audioEl.srcObject = stream

        if (voiceState.selectedOutputId && voiceState.selectedOutputId !== 'default' && typeof audioEl.setSinkId === 'function') {
            audioEl.setSinkId(voiceState.selectedOutputId).catch(() => {})
        }

        audioEl.play().catch(() => {})

        const peerAudioCtx = new (window.AudioContext || window.webkitAudioContext)()
        peerAudioCtx.resume().catch(() => {})

        const source = peerAudioCtx.createMediaStreamSource(stream)
        const analyser = peerAudioCtx.createAnalyser()
        analyser.fftSize = 512
        analyser.smoothingTimeConstant = 0.4

        const dummyDest = peerAudioCtx.createMediaStreamDestination()
        source.connect(analyser)
        analyser.connect(dummyDest)

        peers[peerId].audioEl = audioEl
        peers[peerId].analyser = analyser
        peers[peerId].audioCtx = peerAudioCtx
    }

    if (isInitiator) {
        pc.createOffer().then(offer => {
            return pc.setLocalDescription(offer)
        }).then(() => {
            if (signalingSocket?.readyState === WebSocket.OPEN) {
                signalingSocket.send(JSON.stringify({
                    type: 'offer',
                    offer: pc.localDescription,
                    target: peerId
                }))
            }
        }).catch(() => {})
    }
}

function destroyPeer(peerId) {
    const peer = peers[peerId]
    if (peer) {
        if (peer.pc) peer.pc.close()
        if (peer.audioEl) {
            peer.audioEl.pause()
            peer.audioEl.srcObject = null
        }
        if (peer.analyser) peer.analyser.disconnect()
        if (peer.audioCtx) peer.audioCtx.close()
        delete peers[peerId]
    }
    voiceState.participants = voiceState.participants.filter(p => p.id !== peerId)
}

function monitorVoiceActivity() {
    if (!audioContext) return

    const HYSTERESIS_MS = 300
    let lastUpdate = 0

    const checkActivity = (time) => {
        if (!voiceState.isConnected && !voiceState.isTestingMic) return

        if (time - lastUpdate > 50) {
            const now = Date.now()

            if (localAnalyser) {
                const data = new Uint8Array(localAnalyser.frequencyBinCount)
                localAnalyser.getByteFrequencyData(data)

                let vocalEnergy = 0
                for (let i = 3; i < 32; i++) {
                    vocalEnergy += data[i]
                }
                vocalEnergy /= 29

                if (vocalEnergy < noiseFloor) {
                    noiseFloor = vocalEnergy
                } else {
                    noiseFloor += 0.05
                }

                if (noiseFloor < 5) noiseFloor = 5

                const isCurrentlySpeaking = (vocalEnergy > noiseFloor + 12) && !voiceState.isMuted

                if (isCurrentlySpeaking) {
                    localLastSpokeTime = now
                    if (gateGainNode && audioContext && state.settings.voice_noise_suppression !== false) {
                        gateGainNode.gain.setTargetAtTime(1, audioContext.currentTime, 0.03)
                    }
                } else {
                    if (now - localLastSpokeTime > HYSTERESIS_MS) {
                        if (gateGainNode && audioContext && state.settings.voice_noise_suppression !== false) {
                            gateGainNode.gain.setTargetAtTime(0, audioContext.currentTime, 0.15)
                        }
                    }
                }

                voiceState.localSpeaking = (now - localLastSpokeTime) < HYSTERESIS_MS

                if (voiceState.isTestingMic) {
                    voiceState.testMicVolume = Math.min(Math.round((vocalEnergy / 255) * 100), 100)
                }
            }

            voiceState.participants.forEach(p => {
                const peer = peers[p.id]
                if (peer?.analyser) {
                    const data = new Uint8Array(peer.analyser.frequencyBinCount)
                    peer.analyser.getByteFrequencyData(data)

                    let vocalEnergy = 0
                    for (let i = 3; i < 32; i++) {
                        vocalEnergy += data[i]
                    }
                    vocalEnergy /= 29

                    const isCurrentlySpeaking = vocalEnergy > 15 && !p.muted

                    if (isCurrentlySpeaking) {
                        peer.lastSpokeTime = now
                    }

                    p.speaking = (now - (peer.lastSpokeTime || 0)) < HYSTERESIS_MS
                }
            })
            lastUpdate = time
        }

        vadAnimationId = requestAnimationFrame(checkActivity)
    }
    requestAnimationFrame(checkActivity)
}

export async function inviteToParty(targetUserId) {
    if (signalingSocket?.readyState !== WebSocket.OPEN) return
    try {
        const data = await bridge.partyInvitePrepare()
        signalingSocket.send(JSON.stringify({
            type: 'party-invite',
            target: targetUserId,
            mods: data.mods,
            tunnelUrl: data.tunnel_url
        }))
        showToast(t("K.I.P. Party"), t("Invite sent!"), "success")
    } catch (e) {}
}

export async function acceptPartyInvite() {
    const invite = state.partyInvite
    if (!invite) return
    state.partyInvite = null

    showToast(t("K.I.P. Party"), t("Accepting invite and syncing mods..."), "info")

    let host = '127.0.0.1:8765'
    if (invite.tunnelUrl) {
        const match = invite.tunnelUrl.match(/(?:tcp:\/\/|https:\/\/|wss:\/\/)([a-zA-Z0-9\.\-]+(?::\d+)?)/)
        if (match) host = match[1]
    }

    if (signalingSocket?.readyState === WebSocket.OPEN) {
        signalingSocket.send(JSON.stringify({
            type: 'party-accept',
            target: invite.senderId
        }))
    }

    if (invite.tunnelUrl) {
        try { navigator.clipboard.writeText(invite.tunnelUrl) } catch (e) {}
        showToast(t("K.I.P. Party"), t("Server IP copied to clipboard!"), "success")
    }

    if (invite.mods && invite.mods.length > 0) {
        const issues = invite.mods.map(modName => ({
            action: 'DOWNLOAD',
            target: modName,
            selected: true
        }))
        try {
            const res = await bridge.applyDoctorFixes(issues)
            if (res.success || res.downloaded > 0) {
                showToast(t("Success"), `Sync complete. ${res.downloaded} mods downloaded.`, "success")
            }
        } catch (e) {}
    }

    setTimeout(() => {
        joinVoiceChannel('kip-party', host)
    }, 1000)
}

export function declinePartyInvite() {
    const invite = state.partyInvite
    if (!invite) return
    state.partyInvite = null
    if (signalingSocket?.readyState === WebSocket.OPEN) {
        signalingSocket.send(JSON.stringify({
            type: 'party-decline',
            target: invite.senderId
        }))
    }
}

let gamepadState = { active: false, focusedEl: null, lastInput: 0, raf: null }

export function initGamepadMode() {
    window.addEventListener("gamepadconnected", () => {
        gamepadState.active = true
        if (!gamepadState.raf) gamepadLoop()
    })
    window.addEventListener("gamepaddisconnected", () => {
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

function switchView(dir) {
    const views = state.navItems.map(n => n.id)
    let currIdx = views.indexOf(state.currentView)
    currIdx += dir
    if (currIdx < 0) currIdx = views.length - 1
    if (currIdx >= views.length) currIdx = 0
    state.currentView = views[currIdx]
}

function getInteractables() {
    let root = document.body
    const activeModals = Array.from(document.querySelectorAll('.fixed.z-\\[200\\], .fixed.z-\\[300\\], .fixed.z-\\[10000\\], .fixed.z-\\[99999\\]')).filter(el => {
        const style = window.getComputedStyle(el)
        return style.display !== 'none' && style.opacity !== '0' && style.visibility !== 'hidden' && style.pointerEvents !== 'none' && el.getBoundingClientRect().width > 0
    })

    if (activeModals.length > 0) {
        root = activeModals.sort((a,b) => window.getComputedStyle(b).zIndex - window.getComputedStyle(a).zIndex)[0]
    } else if (state.isNexusOpen) {
        root = document.querySelector('.z-\\[100\\]') || document.body
    }

    return Array.from(root.querySelectorAll('button, input, select, textarea, a, .cursor-pointer, .kip-card-hover')).filter(el => {
        const rect = el.getBoundingClientRect()
        const style = window.getComputedStyle(el)
        if (style.visibility === 'hidden' || style.opacity === '0' || style.display === 'none' || style.pointerEvents === 'none' || el.disabled) return false
        if (state.isBigPicture && el.closest('.titlebar')) return false
        return rect.width > 0 && rect.height > 0
    })
}

function gamepadLoop() {
    if (!gamepadState.active) {
        gamepadState.raf = null
        return
    }

    let gp = null
    const gps = navigator.getGamepads ? navigator.getGamepads() : []
    for (let i = 0; i < gps.length; i++) {
        if (gps[i]) { gp = gps[i]; break }
    }

    if (gp) {
        const now = Date.now()
        let dx = 0, dy = 0
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
                const closeBtns = interactables.filter(b =>
                    b.innerHTML.includes('lucide-x') ||
                    b.textContent.includes('Close') ||
                    b.textContent.includes('Cancel') ||
                    b.textContent.includes('Decline')
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

function moveGamepadFocus(dx, dy) {
    const interactables = getInteractables()
    if (interactables.length === 0) return

    if (!gamepadState.focusedEl || !document.body.contains(gamepadState.focusedEl) || !interactables.includes(gamepadState.focusedEl)) {
        setGamepadFocus(interactables[0])
        return
    }

    const currentRect = gamepadState.focusedEl.getBoundingClientRect()
    const currentCenterX = currentRect.left + currentRect.width / 2
    const currentCenterY = currentRect.top + currentRect.height / 2

    let bestMatch = null
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

function setGamepadFocus(el) {
    if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('gamepad-focus')
    gamepadState.focusedEl = el
    el.classList.add('gamepad-focus')
    el.focus({ preventScroll: true })
    el.scrollIntoView({ behavior: 'auto', block: 'center', inline: 'center' })
}

export { setupTauriListeners }