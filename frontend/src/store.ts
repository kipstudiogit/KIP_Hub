import { ref } from 'vue'
import { bridge, setupTauriListeners } from './bridge'

export * from './types/app'
export * from './types/voice'
export * from './utils/format'
export * from './composables/useToasts'
export * from './composables/useI18n'
export * from './composables/useWindow'
export * from './stores/appState'
export * from './composables/useVoice'
export * from './composables/useGamepad'

export const api = ref(bridge)
export { setupTauriListeners }