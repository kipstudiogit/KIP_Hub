import { state } from '../stores/appState'
import { toggleBigPicture } from './useWindow'

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

function setGamepadFocus(el: HTMLElement): void {
  if (gamepadState.focusedEl) gamepadState.focusedEl.classList.remove('gamepad-focus')
  gamepadState.focusedEl = el
  el.classList.add('gamepad-focus')
  el.focus({ preventScroll: true })
  el.scrollIntoView({ behavior: 'auto', block: 'center', inline: 'center' })
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