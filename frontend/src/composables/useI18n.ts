import { bridge } from '../bridge'
import { state } from '../stores/appState'

export function t(key: string): string {
  if (!key) return ''
  const lang = state.settings.lang
  if (!lang || !state.translations[lang]) return key
  return state.translations[lang][key] || key
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
      state.translations[lang] = {}
    }
  }
}