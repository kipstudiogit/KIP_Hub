<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-extrabold mb-1">{{ t('Support') }}</h2>
      <p class="text-white/50 text-sm">{{ t('Get help from AI or send feedback to developers.') }}</p>
    </div>

    <div class="grid grid-cols-2 gap-6 flex-1 min-h-0 pb-10 stagger-2">
      <div class="kip-card p-8 flex flex-col relative overflow-hidden group hover:border-purple-500/30 transition-colors duration-500">
        <div class="absolute -top-32 -right-32 w-96 h-96 bg-purple-600/10 blur-[100px] rounded-full pointer-events-none transition-all duration-700 group-hover:bg-purple-600/20"></div>

        <div class="flex justify-between items-start mb-6 relative z-10 shrink-0">
          <h3 class="text-2xl font-extrabold flex items-center gap-3 text-white">
            <div class="p-2 bg-purple-500/10 rounded-xl border border-purple-500/20"><Sparkles class="w-6 h-6 text-purple-400" /></div>
            {{ t('AI Support') }}
          </h3>
          <button @click="fetchLatestLog" class="kip-btn-ghost px-4 py-2 text-xs border-white/5 hover:border-white/10 text-white/70 hover:text-white">
            <FileTerminal class="w-4 h-4" /> {{ t('Load Latest Log') }}
          </button>
        </div>

        <p class="text-white/50 text-sm mb-4 shrink-0 relative z-10">{{ t('Gemini AI will analyze your crash logs.') }}</p>

        <textarea v-model="state.aiInputText" class="kip-input h-40 font-mono text-xs resize-none custom-scroll relative z-10 mb-5 shadow-inner focus:border-purple-500" :placeholder="t('Paste crash log snippet here...')"></textarea>

        <button @click="askAI" :disabled="isAiLoading || !state.aiInputText.trim()" class="kip-btn-primary py-4 bg-purple-500 hover:bg-purple-400 text-white border-purple-400 shadow-[0_0_20px_rgba(168,85,247,0.3)] relative z-10 shrink-0 text-base">
          <Loader v-if="isAiLoading" class="w-5 h-5 animate-spin" />
          <BrainCircuit v-else class="w-5 h-5" />
          {{ isAiLoading ? t('Analyzing...') : t('Analyze') }}
        </button>

        <div class="flex-1 mt-6 overflow-y-auto custom-scroll relative z-10 bg-black/40 border border-white/5 rounded-2xl shadow-inner">
          <div class="w-full h-full p-6" v-html="aiResponseHtml"></div>
        </div>
      </div>

      <div class="flex flex-col gap-6 relative z-10 min-h-0">
        <div class="kip-card p-6 border-indigo-500/20 bg-indigo-500/5 relative overflow-hidden group">
          <div class="absolute -bottom-10 -right-10 w-32 h-32 bg-indigo-600/20 blur-[50px] rounded-full pointer-events-none"></div>
          <h4 class="text-xs font-bold text-indigo-400 uppercase tracking-wider mb-4 flex items-center gap-2">
            <Cpu class="w-4 h-4" /> System Telemetry
          </h4>
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-black/40 p-3 rounded-xl border border-white/5 shadow-inner flex flex-col justify-center">
              <span class="text-[9px] text-white/40 uppercase tracking-widest font-bold mb-1">Operating System</span>
              <span class="text-sm font-bold text-white truncate">{{ sysInfo.os }}</span>
            </div>
            <div class="bg-black/40 p-3 rounded-xl border border-white/5 shadow-inner flex flex-col justify-center">
              <span class="text-[9px] text-white/40 uppercase tracking-widest font-bold mb-1">Processor (CPU)</span>
              <span class="text-sm font-bold text-white truncate">{{ sysInfo.cpu }}</span>
            </div>
            <div class="bg-black/40 p-3 rounded-xl border border-white/5 shadow-inner flex flex-col justify-center">
              <span class="text-[9px] text-white/40 uppercase tracking-widest font-bold mb-1">Available RAM</span>
              <span class="text-sm font-bold text-white truncate font-mono">{{ sysInfo.ram }} GB</span>
            </div>
            <div class="bg-black/40 p-3 rounded-xl border border-white/5 shadow-inner flex flex-col justify-center">
              <span class="text-[9px] text-white/40 uppercase tracking-widest font-bold mb-1">Java Version</span>
              <span class="text-sm font-bold text-white truncate font-mono">{{ sysInfo.java }}</span>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 flex flex-col relative overflow-hidden group hover:border-indigo-500/30 transition-colors duration-500 flex-1">
          <div class="absolute -bottom-32 -left-32 w-96 h-96 bg-indigo-600/10 blur-[100px] rounded-full pointer-events-none transition-all duration-700 group-hover:bg-indigo-600/20"></div>

          <div class="flex justify-between items-start mb-6 relative z-10 shrink-0">
            <h3 class="text-2xl font-extrabold flex items-center gap-3 text-white">
              <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20"><Bug class="w-6 h-6 text-indigo-400" /></div>
              {{ t('Bug Report') }}
            </h3>
          </div>

          <p class="text-white/50 text-sm mb-6 shrink-0 relative z-10">{{ t('Send feedback directly to the developers. System telemetry is attached automatically.') }}</p>

          <div class="flex-1 flex flex-col gap-5 relative z-10 min-h-0">
            <textarea v-model="bugInputText" class="kip-input flex-1 resize-none custom-scroll shadow-inner text-sm leading-relaxed" :placeholder="t('Describe the issue...')"></textarea>

            <button @click="sendBugReport" :disabled="isBugSending || !bugInputText.trim()" class="kip-btn-primary py-4 text-base shadow-[0_0_20px_rgba(99,102,241,0.3)] shrink-0">
              <Loader v-if="isBugSending" class="w-5 h-5 animate-spin" />
              <Send v-else class="w-5 h-5" />
              {{ isBugSending ? t('Sending...') : t('Submit Report') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Sparkles, Send, Loader, Bug, BrainCircuit, FileTerminal, Cpu } from 'lucide-vue-next'
import { state, t, showToast, sanitizeHTML } from '@/store'
import { invokeSafe } from '@/bridge'

interface SysInfoParsed {
  os: string
  cpu: string
  ram: string
  java: string
}

interface AiAnalysisResult {
  success: boolean
  answer: string
}

const isAiLoading = ref<boolean>(false)
const aiResponseHtml = ref<string>(
  '<div class="h-full flex flex-col items-center justify-center text-white/30 gap-4"><svg class="w-12 h-12 opacity-50" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 8V4H8"/><rect width="16" height="12" x="4" y="8" rx="2"/><path d="M2 14h2"/><path d="M20 14h2"/><path d="M15 13v2"/><path d="M9 13v2"/></svg><p class="text-sm font-medium tracking-wide">Waiting for input...</p></div>'
)

const bugInputText = ref<string>('')
const isBugSending = ref<boolean>(false)
const sysInfo = ref<SysInfoParsed>({
  os: 'Loading...',
  cpu: 'Loading...',
  ram: '...',
  java: '...',
})

const loadSysInfo = async (): Promise<void> => {
  try {
    const rawData = await invokeSafe<string>('get_sys_info')
    const lines = rawData.split('\n')
    lines.forEach((line) => {
      if (line.startsWith('OS:')) sysInfo.value.os = line.replace('OS:', '').trim()
      if (line.startsWith('CPU:')) sysInfo.value.cpu = line.replace('CPU:', '').trim()
      if (line.startsWith('RAM:')) sysInfo.value.ram = line.replace('RAM:', '').replace('GB', '').trim()
      if (line.startsWith('Java:')) sysInfo.value.java = line.replace('Java:', '').trim()
    })
  } catch {
    // Retains loading state on IPC failure
  }
}

const fetchLatestLog = async (): Promise<void> => {
  try {
    const logData = await invokeSafe<string>('get_console_logs')
    if (logData) {
      state.aiInputText = logData
      showToast(t('Success'), t('Loaded the latest log tail.'), 'success')
    } else {
      showToast(t('Error'), t('Failed to load log file.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Failed to communicate with backend.'), 'danger')
  }
}

const askAI = async (): Promise<void> => {
  const cleanInput = state.aiInputText.trim()
  if (!cleanInput) return

  if (
    !state.settings.ai_api_key &&
    !state.settings.openai_api_key &&
    !state.settings.anthropic_api_key &&
    state.settings.ai_provider !== 'ollama'
  ) {
    showToast(t('Error'), t('AI API Key is required. Set it in Settings.'), 'danger')
    return
  }

  isAiLoading.value = true
  aiResponseHtml.value =
    '<div class="h-full flex items-center justify-center gap-4 text-purple-400 font-bold animate-pulse tracking-wide"><svg class="w-8 h-8" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2"/><path d="M15 20v2"/><path d="M2 15h2"/><path d="M2 9h2"/><path d="M20 15h2"/><path d="M20 9h2"/><path d="M9 2v2"/><path d="M9 20v2"/></svg> Neural Core is thinking...</div>'

  try {
    const res = await invokeSafe<AiAnalysisResult>('analyze_crash_ai', {
      logSnippet: cleanInput,
    })

    if (res && res.success) {
      const safeText = sanitizeHTML(res.answer)
      const htmlAns = safeText
        .replace(/\*\*(.*?)\*\*/g, '<span class="text-white font-extrabold">$1</span>')
        .replace(/\n/g, '<br>')

      aiResponseHtml.value = `
        <div class="flex items-start gap-4">
          <div class="p-2.5 bg-purple-500/20 border border-purple-500/30 rounded-xl shrink-0">
            <svg class="w-6 h-6 text-purple-400" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/><path d="M5 3v4"/><path d="M3 5h4"/></svg>
          </div>
          <div class="text-sm text-white/80 leading-relaxed font-medium markdown-body">${htmlAns}</div>
        </div>`
    } else {
      aiResponseHtml.value = `<div class="text-red-400 border border-red-500/30 bg-red-500/10 p-5 rounded-2xl font-bold flex items-center gap-3"><svg class="w-6 h-6 shrink-0" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg> Error: ${sanitizeHTML(res?.answer || 'Failed to analyze.')}</div>`
    }
  } catch {
    aiResponseHtml.value = `<div class="text-red-400 border border-red-500/30 bg-red-500/10 p-5 rounded-2xl font-bold">Backend Error</div>`
  } finally {
    isAiLoading.value = false
  }
}

const sendBugReport = async (): Promise<void> => {
  const cleanReport = bugInputText.value.trim()
  if (!cleanReport || isBugSending.value) return
  isBugSending.value = true

  try {
    const success = await invokeSafe<boolean>('send_bug_report', {
      reportText: cleanReport,
    })
    if (success) {
      showToast(t('Sent'), t('Thank you for your feedback!'), 'success')
      bugInputText.value = ''
    } else {
      showToast(t('Error'), t('Failed to send report.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Failed to send report.'), 'danger')
  } finally {
    isBugSending.value = false
  }
}

onMounted(() => {
  loadSysInfo()
})
</script>