import { ref, reactive } from 'vue'
import { state } from '../stores/appState'
import { toggleBigPicture } from './useWindow'

export interface ConnectedGamepadInfo {
  id: string
  index: number
  connected: boolean
  modelName: string
  batteryPercent: number | null
}

export const activeGamepad = ref<ConnectedGamepadInfo | null>(null)
export const isGamepadActive = ref<boolean>(false)

let audioContext: AudioContext | null = null

function getAudioContext(): AudioContext {
  if (!audioContext) {
    const AudioClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext
    audioContext = new AudioClass({ latencyHint: 'interactive' })
  }
  if (audioContext.state === 'suspended') {
    audioContext.resume().catch(() => {})
  }
  return audioContext
}

export function playNavSound(): void {
  try {
    const ctx = getAudioContext()
    const osc = ctx.createOscillator()
    const gain = ctx.createGain()
    const now = ctx.currentTime

    osc.type = 'sine'
    osc.frequency.setValueAtTime(380, now)
    osc.frequency.exponentialRampToValueAtTime(180, now + 0.04)

    gain.gain.setValueAtTime(0.08, now)
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04)

    osc.connect(gain)
    gain.connect(ctx.destination)
    osc.start(now)
    osc.stop(now + 0.04)
  } catch {}
}

export function playConfirmSound(): void {
  try {
    const ctx = getAudioContext()
    const now = ctx.currentTime

    const osc1 = ctx.createOscillator()
    const osc2 = ctx.createOscillator()
    const gain = ctx.createGain()

    osc1.type = 'triangle'
    osc2.type = 'sine'

    osc1.frequency.setValueAtTime(523.25, now) // C5
    osc2.frequency.setValueAtTime(659.25, now) // E5

    gain.gain.setValueAtTime(0.12, now)
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.12)

    osc1.connect(gain)
    osc2.connect(gain)
    gain.connect(ctx.destination)

    osc1.start(now)
    osc2.start(now)
    osc1.stop(now + 0.12)
    osc2.stop(now + 0.12)
  } catch {}
}

export function playCancelSound(): void {
  try {
    const ctx = getAudioContext()
    const osc = ctx.createOscillator()
    const gain = ctx.createGain()
    const now = ctx.currentTime

    osc.type = 'sawtooth'
    osc.frequency.setValueAtTime(280, now)
    osc.frequency.exponentialRampToValueAtTime(140, now + 0.08)

    gain.gain.setValueAtTime(0.08, now)
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08)

    osc.connect(gain)
    gain.connect(ctx.destination)
    osc.start(now)
    osc.stop(now + 0.08)
  } catch {}
}

export function playLaunchSound(): void {
  try {
    const ctx = getAudioContext()
    const now = ctx.currentTime

    const osc = ctx.createOscillator()
    const sub = ctx.createOscillator()
    const gain = ctx.createGain()

    osc.type = 'sine'
    sub.type = 'triangle'

    osc.frequency.setValueAtTime(80, now)
    osc.frequency.exponentialRampToValueAtTime(580, now + 0.45)

    sub.frequency.setValueAtTime(45, now)
    sub.frequency.exponentialRampToValueAtTime(180, now + 0.45)

    gain.gain.setValueAtTime(0.2, now)
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.45)

    osc.connect(gain)
    sub.connect(gain)
    gain.connect(ctx.destination)

    osc.start(now)
    sub.start(now)
    osc.stop(now + 0.45)
    sub.stop(now + 0.45)
  } catch {}
}

export function triggerHaptic(duration = 75, weak = 0.5, strong = 0.3): void {
  try {
    const gamepads = navigator.getGamepads ? navigator.getGamepads() : []
    for (const gp of gamepads) {
      if (gp) {
        const actuator = (gp as unknown as { vibrationActuator?: { playEffect: (type: string, opts: Record<string, number>) => Promise<void> } }).vibrationActuator
        if (actuator?.playEffect) {
          actuator.playEffect('dual-rumble', {
            startDelay: 0,
            duration,
            weakMagnitude: weak,
            strongMagnitude: strong,
          }).catch(() => {})
        }
      }
    }
  } catch {}
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
  raf: null,
}

function parseControllerModel(id: string): string {
  const lower = id.toLowerCase()
  if (lower.includes('045e') || lower.includes('xbox')) return 'Xbox Wireless Controller'
  if (lower.includes('054c') || lower.includes('dualsense') || lower.includes('dualshock')) return 'PlayStation DualSense'
  if (lower.includes('057e') || lower.includes('nintendo') || lower.includes('pro controller')) return 'Nintendo Switch Pro'
  return id.split('(')[0]?.trim() || 'Console Gamepad'
}

function switchView(dir: number): void {
  const views = state.navItems.map((n) => n.id)
  let currIdx = views.indexOf(state.currentView)
  currIdx += dir
  if (currIdx < 0) currIdx = views.length - 1
  if (currIdx >= views.length) currIdx = 0
  state.currentView = views[currIdx]
  playNavSound()
  triggerHaptic(40, 0.3, 0.1)
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
      'button, input, select, textarea, a, .cursor-pointer, .kip-card-hover, [data-gamepad-action]'
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

function setGamepadFocus(el: HTMLElement): void {
  if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('gamepad-focus')
  gamepadState.focusedEl = el
  el.classList.add('gamepad-focus')
  el.focus({ preventScroll: true })
  el.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'center' })
  playNavSound()
  triggerHaptic(30, 0.25, 0.05)
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
    const btnX = gp.buttons[2]?.pressed
    const btnY = gp.buttons[3]?.pressed

    const hasAction = dx !== 0 || dy !== 0 || btnL1 || btnR1 || btnA || btnB || btnX || btnY

    if (hasAction && now - gamepadState.lastInput > 160) {
      if (dx !== 0 || dy !== 0) {
        moveGamepadFocus(dx, dy)
        gamepadState.lastInput = now
      } else if (btnL1) {
        switchView(-1)
        gamepadState.lastInput = now + 180
      } else if (btnR1) {
        switchView(1)
        gamepadState.lastInput = now + 180
      } else if (btnA) {
        if (gamepadState.focusedEl && document.body.contains(gamepadState.focusedEl)) {
          playConfirmSound()
          triggerHaptic(80, 0.6, 0.4)
          gamepadState.focusedEl.click()
          gamepadState.focusedEl.classList.add('scale-95')
          setTimeout(() => {
            if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('scale-95')
          }, 100)
          gamepadState.lastInput = now + 220
        }
      } else if (btnB) {
        playCancelSound()
        triggerHaptic(60, 0.4, 0.2)
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
        gamepadState.lastInput = now + 240
      } else if (btnY) {
        // Quick Ignition Key
        playLaunchSound()
        triggerHaptic(180, 0.8, 0.9)
        const launchBtn = document.querySelector<HTMLButtonElement>('[data-gamepad-action="ignite"]')
        if (launchBtn) {
          launchBtn.click()
        } else {
          state.currentView = 'launcher'
        }
        gamepadState.lastInput = now + 350
      } else if (btnX) {
        // Quick Doctor Diagnostic Key
        playConfirmSound()
        triggerHaptic(70, 0.4, 0.3)
        state.currentView = 'tools'
        gamepadState.lastInput = now + 250
      }
    }
  }

  gamepadState.raf = requestAnimationFrame(gamepadLoop)
}

export function initGamepadMode(): void {
  window.addEventListener('gamepadconnected', (e: GamepadEvent) => {
    isGamepadActive.value = true
    gamepadState.active = true

    const gp = e.gamepad
    activeGamepad.value = {
      id: gp.id,
      index: gp.index,
      connected: true,
      modelName: parseControllerModel(gp.id),
      batteryPercent: null,
    }

    triggerHaptic(120, 0.5, 0.3)
    playConfirmSound()

    if (!gamepadState.raf) gamepadLoop()
  })

  window.addEventListener('gamepaddisconnected', () => {
    const gps = navigator.getGamepads ? navigator.getGamepads() : []
    let anyActive = false
    for (let i = 0; i < gps.length; i++) {
      if (gps[i]) {
        anyActive = true
        activeGamepad.value = {
          id: gps[i]!.id,
          index: gps[i]!.index,
          connected: true,
          modelName: parseControllerModel(gps[i]!.id),
          batteryPercent: null,
        }
        break
      }
    }

    if (!anyActive) {
      isGamepadActive.value = false
      activeGamepad.value = null
      gamepadState.active = false
      if (gamepadState.focusedEl) {
        gamepadState.focusedEl.classList.remove('gamepad-focus')
        gamepadState.focusedEl = null
      }
    }
  })
}