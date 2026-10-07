import { invoke } from '@tauri-apps/api/core'
import { state } from '../stores/appState'

export function t(key: string): string {
  if (!key) return ''
  const lang = state.settings.lang || 'en'
  const activeDict = state.translations[lang]
  if (activeDict && activeDict[key]) {
    return activeDict[key]
  }
  const enDict = state.translations['en']
  if (enDict && enDict[key]) {
    return enDict[key]
  }
  return key
}

export async function setLanguage(lang: string): Promise<void> {
  state.settings.lang = lang
  await loadTranslations(lang)
}

export async function loadTranslations(targetLang?: string): Promise<void> {
  const lang = targetLang || state.settings.lang || 'en'
  const supported = ['en', 'ru', 'es', 'de', 'zh', 'fr', 'pt', 'ja', 'ko']
  const activeLang = supported.includes(lang) ? lang : 'en'

  state.settings.lang = activeLang

  try {
    const dict = await invoke<Record<string, string>>('get_translations', { lang: activeLang })
    if (dict && Object.keys(dict).length > 0) {
      state.translations[activeLang] = dict
    }
  } catch {
    state.translations[activeLang] = {}
  }

  if (activeLang !== 'en' && (!state.translations['en'] || Object.keys(state.translations['en']).length === 0)) {
    try {
      const enDict = await invoke<Record<string, string>>('get_translations', { lang: 'en' })
      if (enDict && Object.keys(enDict).length > 0) {
        state.translations['en'] = enDict
      }
    } catch {
      state.translations['en'] = {}
    }
  }
}