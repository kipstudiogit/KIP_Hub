<template>
  <div class="h-full flex flex-col custom-scroll overflow-y-auto pr-2 pb-10" :class="{'justify-center items-center text-center overflow-hidden pr-0 pb-0': state.isMiniMode}">
    <template v-if="!state.isMiniMode">
      <div class="kip-card p-10 mb-8 relative overflow-hidden group shrink-0 stagger-1">
        <div class="absolute top-0 right-0 w-[500px] h-[500px] bg-gradient-to-br from-indigo-500/10 to-purple-500/10 blur-[100px] rounded-full group-hover:from-indigo-500/20 group-hover:to-purple-500/20 transition-all duration-700 pointer-events-none"></div>
        <div class="absolute -bottom-20 -left-20 w-64 h-64 bg-emerald-500/10 blur-[80px] rounded-full pointer-events-none transition-all duration-700"></div>

        <div class="relative z-10 flex flex-col gap-2">
          <span class="text-indigo-400 font-bold tracking-[0.2em] text-xs uppercase">{{ currentDate }}</span>
          <h1 class="text-5xl font-extrabold bg-clip-text text-transparent bg-gradient-to-r from-white to-white/70 tracking-tight flex items-center gap-3">
            {{ t(state.greeting) }} <span class="animate-[pulse_3s_ease-in-out_infinite] inline-block">👋</span>
          </h1>
          <p class="text-white/50 text-lg mt-2 font-medium tracking-wide">{{ t('Welcome to your ultimate manager.') }}</p>
        </div>
      </div>

      <div class="grid grid-cols-4 gap-5 mb-8 shrink-0 stagger-2">
        <div class="kip-card kip-card-hover p-6 flex flex-col justify-between h-32 relative overflow-hidden group">
          <div class="absolute -right-4 -top-4 w-16 h-16 bg-indigo-500/20 blur-2xl rounded-full group-hover:bg-indigo-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('MC Size') }}</span>
            <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20"><HardDrive class="text-indigo-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-white relative z-10 tracking-tight">{{ state.stats.size }}</div>
        </div>

        <div @click="state.currentView = 'worlds'" class="kip-card kip-card-hover p-6 flex flex-col justify-between h-32 relative overflow-hidden group cursor-pointer border-transparent hover:border-emerald-500/30">
          <div class="absolute -right-4 -top-4 w-16 h-16 bg-emerald-500/20 blur-2xl rounded-full group-hover:bg-emerald-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('Worlds') }}</span>
            <div class="p-2 bg-emerald-500/10 rounded-xl border border-emerald-500/20"><Globe class="text-emerald-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-white relative z-10 flex items-baseline gap-2 tracking-tight">
            {{ state.stats.saves }}
            <ArrowRight class="w-4 h-4 text-emerald-400 opacity-0 -translate-x-4 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
          </div>
        </div>

        <div class="kip-card kip-card-hover p-6 flex flex-col justify-between h-32 relative overflow-hidden group border-transparent hover:border-purple-500/30">
          <div class="absolute -right-4 -top-4 w-16 h-16 bg-purple-500/20 blur-2xl rounded-full group-hover:bg-purple-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('Playtime') }}</span>
            <div class="p-2 bg-purple-500/10 rounded-xl border border-purple-500/20"><Clock class="text-purple-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-indigo-400 relative z-10 truncate tracking-tight">{{ state.stats.playtime }}</div>
        </div>

        <div class="kip-card kip-card-hover p-6 flex flex-col justify-between h-32 relative overflow-hidden group border-transparent hover:border-amber-500/30">
          <div class="absolute -right-4 -top-4 w-16 h-16 bg-amber-500/20 blur-2xl rounded-full group-hover:bg-amber-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">Java Runtime</span>
            <div class="p-2 bg-amber-500/10 rounded-xl border border-amber-500/20"><Coffee class="text-amber-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-white relative z-10 truncate tracking-tight">{{ state.stats.java }}</div>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-5 shrink-0 stagger-3">
        <div class="col-span-2 kip-card p-8 border border-blue-500/20 relative overflow-hidden group transition-all duration-500">
          <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500 to-cyan-500 opacity-50 group-hover:opacity-100 transition-opacity duration-500"></div>
          <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-blue-500/10 blur-[80px] rounded-full pointer-events-none group-hover:bg-blue-500/20 transition-all duration-500"></div>

          <div class="flex items-center gap-3 mb-6 relative z-10">
            <div class="p-2.5 bg-blue-500/10 rounded-xl border border-blue-500/20"><Newspaper class="w-5 h-5 text-blue-400" /></div>
            <h3 class="text-xl font-bold text-white">{{ t('Project News') }}</h3>
            <span class="px-2.5 py-1 bg-blue-500/20 text-blue-400 text-[10px] font-bold uppercase tracking-widest rounded-lg border border-blue-500/30">Latest</span>
          </div>

          <div v-if="state.newsText" class="text-sm text-white/70 leading-relaxed relative z-10 markdown-body max-w-none" v-html="sanitizeHTML(state.newsText)"></div>
          <div v-else class="text-sm text-white/30 relative z-10 flex items-center gap-3 font-medium">
            <Loader class="w-4 h-4 animate-spin text-blue-500" /> Fetching latest broadcasts...
          </div>
        </div>

        <div class="col-span-1 kip-card p-8 flex flex-col justify-center items-center text-center relative overflow-hidden transition-all duration-500" :class="state.isMcRunning ? 'border-emerald-500/40 bg-emerald-900/10 shadow-[0_0_30px_rgba(16,185,129,0.1)]' : 'border-white/10'">
          <div class="absolute inset-0 flex items-center justify-center pointer-events-none z-0">
            <div v-if="state.isMcRunning" class="w-48 h-48 border border-emerald-500/20 rounded-full animate-[ping_3s_cubic-bezier(0,0,0.2,1)_infinite]"></div>
            <div v-if="state.isMcRunning" class="w-32 h-32 border border-emerald-500/30 rounded-full animate-[ping_2s_cubic-bezier(0,0,0.2,1)_infinite]"></div>
          </div>

          <div class="relative z-10 flex flex-col items-center">
            <div class="w-20 h-20 rounded-3xl flex items-center justify-center mb-6 transition-all duration-500 shadow-2xl relative" :class="state.isMcRunning ? 'bg-emerald-500/20 border border-emerald-500/50 shadow-[0_0_20px_rgba(16,185,129,0.5)]' : 'bg-black/50 border border-white/10'">
              <div v-if="state.isMcRunning" class="absolute inset-0 bg-emerald-500/20 blur-xl rounded-3xl animate-pulse"></div>
              <Rocket class="w-10 h-10 relative z-10" :class="state.isMcRunning ? 'text-emerald-400 animate-bounce' : 'text-white/30'" />
            </div>

            <h3 class="text-xl font-extrabold mb-2">{{ t('System Status') }}</h3>
            <p class="text-[10px] font-bold uppercase tracking-widest px-4 py-1.5 rounded-full border transition-all duration-500" :class="state.isMcRunning ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30 shadow-[0_0_10px_rgba(52,211,153,0.3)]' : 'bg-white/5 text-white/50 border-white/10'">
              {{ t(state.mcStatusText) }}
            </p>

            <button v-if="!state.isMcRunning" @click="state.currentView = 'launcher'" class="mt-8 kip-btn-ghost px-6 py-2.5 text-xs tracking-wide">
              <Play class="w-3.5 h-3.5" /> Go to Launcher
            </button>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="kip-card p-8 w-full max-w-sm flex flex-col justify-center items-center text-center relative overflow-hidden transition-all duration-500" :class="state.isMcRunning ? 'border-emerald-500/40 bg-emerald-900/10 shadow-[0_0_40px_rgba(16,185,129,0.15)]' : 'border-indigo-500/30 shadow-[0_0_40px_rgba(99,102,241,0.1)]'">
        <div class="absolute -top-20 -left-20 w-48 h-48 blur-[80px] rounded-full pointer-events-none transition-colors duration-500" :class="state.isMcRunning ? 'bg-emerald-500/20' : 'bg-indigo-500/20'"></div>

        <div class="relative w-24 h-24 mb-6 flex items-center justify-center">
          <div class="absolute inset-0 border-2 rounded-full border-dashed animate-[spin_10s_linear_infinite]" :class="state.isMcRunning ? 'border-emerald-500/30' : 'border-indigo-500/30'"></div>
          <div class="absolute inset-2 border-2 rounded-full animate-[spin_5s_linear_infinite_reverse]" :class="state.isMcRunning ? 'border-emerald-500/20' : 'border-indigo-500/20'"></div>
          <Hexagon class="w-10 h-10 transition-colors duration-500" :class="state.isMcRunning ? 'text-emerald-400 animate-pulse' : 'text-indigo-400'" />
        </div>

        <h3 class="font-extrabold text-2xl tracking-widest mb-3 uppercase">K.I.P. Engine</h3>

        <div class="flex items-center gap-2 px-4 py-2 rounded-full border bg-black/40 mb-8 transition-colors duration-500" :class="state.isMcRunning ? 'border-emerald-500/30' : 'border-white/10'">
          <span class="w-2 h-2 rounded-full" :class="state.isMcRunning ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-white/30'"></span>
          <span class="text-[10px] font-bold uppercase tracking-widest" :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/50'">{{ t(state.mcStatusText) }}</span>
        </div>

        <button @click="toggleMiniMode" class="kip-btn-ghost w-full py-3.5 text-xs tracking-wider">
          <Maximize2 class="w-4 h-4" /> {{ t('Expand Dashboard') }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  HardDrive,
  Globe,
  Clock,
  Hexagon,
  ArrowRight,
  Coffee,
  Newspaper,
  Rocket,
  Play,
  Maximize2,
  Loader,
} from 'lucide-vue-next'
import { state, t, sanitizeHTML, toggleMiniMode } from '@/store'

const currentDate = computed<string>(() => {
  try {
    const lang = state.settings.lang || 'en'
    const localeMap: Record<string, string> = {
      zh: 'zh-CN',
      pt: 'pt-BR',
      ja: 'ja-JP',
      ko: 'ko-KR',
      ru: 'ru-RU',
      es: 'es-ES',
      de: 'de-DE',
      fr: 'fr-FR',
      it: 'it-IT',
      pl: 'pl-PL',
      tr: 'tr-TR',
    }
    const resolvedLocale = localeMap[lang] || lang
    return new Date().toLocaleDateString(resolvedLocale, {
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    })
  } catch {
    return new Date().toLocaleDateString('en-US', {
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    })
  }
})
</script>