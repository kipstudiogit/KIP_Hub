<template>
  <div class="h-full overflow-y-auto custom-scroll pr-2 pb-10">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-extrabold mb-1">{{ t('Network & Identity') }}</h2>
      <p class="text-white/50 text-sm">{{ t('Inspect servers, manage panels, and view 3D skins.') }}</p>
    </div>

    <div class="kip-card p-8 mb-6 relative overflow-hidden flex flex-col group transition-colors duration-500 stagger-1" :class="voiceState.isConnected ? 'border-emerald-500/30 shadow-[0_0_40px_rgba(16,185,129,0.15)]' : 'border-white/5 hover:border-emerald-500/30 hover:shadow-[0_0_30px_rgba(16,185,129,0.1)]'">
      <div class="absolute -right-20 -top-20 w-64 h-64 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-emerald-500/20 transition-all duration-700"></div>

      <div class="flex justify-between items-center mb-6 relative z-10">
        <h3 class="text-xl font-bold flex items-center gap-3 transition-colors" :class="voiceState.isConnected ? 'text-emerald-400' : 'text-white'">
          <div class="p-2 rounded-xl border transition-colors" :class="voiceState.isConnected ? 'bg-emerald-500/10 border-emerald-500/20' : 'bg-white/5 border-white/10'">
            <Mic class="w-5 h-5 transition-colors" :class="voiceState.isConnected ? 'animate-pulse text-emerald-400' : 'text-white/50'" />
          </div>
          K.I.P. Connect
        </h3>
        <div v-if="voiceState.isConnected" class="flex items-center gap-3">
          <div class="flex items-center gap-2 bg-black/40 border border-emerald-500/20 px-4 py-2 rounded-xl">
            <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.8)] animate-pulse"></span>
            <span class="text-xs font-mono font-bold text-white tracking-widest">{{ voiceState.channelId }}</span>
          </div>
          <button @click="toggleVoiceSettings" class="kip-btn-ghost p-2 text-emerald-400/70 hover:text-emerald-400 border-emerald-500/20 hover:border-emerald-500/50 hover:bg-emerald-500/10" title="Voice Settings">
            <Settings class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="relative z-10">
        <div v-if="!voiceState.isConnected" class="flex flex-col gap-4 max-w-2xl">
          <p class="text-sm text-white/50 leading-relaxed font-medium">
            {{ t('Create or join a voice room instantly. Type a name or generate a random one to share with friends!') }}
          </p>
          <div class="flex gap-3">
            <div class="relative flex-1">
              <Hash class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
              <input v-model="connectChannel" @keyup.enter="handleJoinVoice" type="text" :placeholder="t('Room name (e.g. survival)')" class="kip-input pl-11 pr-12">
              <button @click="connectChannel = generateRandomRoom()" class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-white/40 hover:text-emerald-400 transition rounded-lg hover:bg-white/5" :title="t('Generate Random Room')">
                <Dices class="w-4 h-4" />
              </button>
            </div>
            <button @click="handleJoinVoice" class="kip-btn-primary px-8 py-3 bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)]">
              <PhoneCall class="w-4 h-4 fill-current" /> {{ connectChannel ? t('Join Room') : t('Quick Start') }}
            </button>
          </div>
        </div>

        <div v-else class="flex flex-col gap-5">
          <transition name="fade">
            <div v-if="voiceState.showSettings" class="bg-black/60 border border-white/10 rounded-2xl p-6 mb-2 flex flex-col gap-5 shadow-inner">
              <div class="grid grid-cols-2 gap-6">
                <div>
                  <label class="text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2 block">Microphone</label>
                  <select v-model="voiceState.selectedInputId" @change="setAudioInput(voiceState.selectedInputId)" class="kip-input py-2.5 text-xs appearance-none cursor-pointer">
                    <option value="default">Default Device</option>
                    <option v-for="d in voiceState.inputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Microphone ' + d.deviceId.substring(0,4) }}</option>
                  </select>
                </div>
                <div>
                  <label class="text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2 block">Output / Speakers</label>
                  <select v-model="voiceState.selectedOutputId" @change="setAudioOutput(voiceState.selectedOutputId)" class="kip-input py-2.5 text-xs appearance-none cursor-pointer">
                    <option value="default">Default Device</option>
                    <option v-for="d in voiceState.outputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Speaker ' + d.deviceId.substring(0,4) }}</option>
                  </select>
                </div>
              </div>
              <div class="pt-5 border-t border-white/10 flex items-center justify-between gap-5">
                <button @click="toggleMicTest" class="px-5 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider transition border" :class="voiceState.isTestingMic ? 'bg-red-500/10 text-red-400 hover:bg-red-500/20 border-red-500/30' : 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 border-emerald-500/30'">
                  {{ voiceState.isTestingMic ? 'Stop Test' : 'Test Mic' }}
                </button>
                <div class="flex-1 h-2.5 bg-black/60 rounded-full overflow-hidden border border-white/5 relative shadow-inner">
                  <div class="absolute top-0 left-0 h-full bg-emerald-400 transition-all duration-75 shadow-[0_0_10px_rgba(52,211,153,0.8)]" :style="{ width: voiceState.testMicVolume + '%' }"></div>
                </div>
              </div>
            </div>
          </transition>

          <div class="grid grid-cols-3 gap-4">
            <div class="bg-black/40 border rounded-2xl p-4 flex flex-col gap-3 transition-colors duration-300" :class="voiceState.localSpeaking ? 'border-emerald-500/50 bg-emerald-500/10 shadow-[inset_0_0_20px_rgba(16,185,129,0.1)]' : 'border-white/5'">
              <div class="flex items-center gap-3">
                <img :src="getAvatarUrl(state.settings.ms_name || 'Guest')" class="w-10 h-10 rounded-xl object-cover bg-black/60 p-0.5 transition-colors duration-300" :class="voiceState.localSpeaking ? 'border-2 border-emerald-400 shadow-[0_0_15px_rgba(52,211,153,0.6)]' : 'border border-white/10'">
                <div class="flex-1 min-w-0">
                  <h4 class="font-bold text-sm truncate transition-colors duration-300" :class="voiceState.localSpeaking ? 'text-emerald-400' : 'text-white'">{{ state.settings.ms_name || 'Guest' }} (You)</h4>
                  <div class="flex items-center gap-1 mt-1.5 h-3">
                    <template v-if="voiceState.localSpeaking">
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.8s_ease-in-out_infinite]" style="height: 100%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.9s_ease-in-out_infinite_0.1s]" style="height: 60%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.7s_ease-in-out_infinite_0.2s]" style="height: 80%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_1.0s_ease-in-out_infinite_0.3s]" style="height: 40%;"></span>
                    </template>
                    <template v-else>
                      <span class="w-6 h-0.5 bg-white/20 rounded-full"></span>
                    </template>
                  </div>
                </div>
                <div class="flex flex-col gap-1.5 shrink-0">
                  <button @click="toggleMute" class="p-1.5 rounded-lg transition" :class="voiceState.isMuted ? 'bg-red-500/20 text-red-400' : 'bg-white/5 hover:bg-white/10 text-white/70 hover:text-white'">
                    <MicOff v-if="voiceState.isMuted" class="w-3.5 h-3.5" />
                    <Mic v-else class="w-3.5 h-3.5" />
                  </button>
                  <button @click="toggleDeafen" class="p-1.5 rounded-lg transition" :class="voiceState.isDeafened ? 'bg-red-500/20 text-red-400' : 'bg-white/5 hover:bg-white/10 text-white/70 hover:text-white'">
                    <Headphones class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>

            <div v-for="p in voiceState.participants" :key="p.id" class="bg-black/40 border rounded-2xl p-4 flex flex-col gap-3 transition-colors duration-300" :class="p.speaking ? 'border-emerald-500/50 bg-emerald-500/10 shadow-[inset_0_0_20px_rgba(16,185,129,0.1)]' : 'border-white/5'">
              <div class="flex items-center gap-3">
                <img :src="getAvatarUrl(p.name)" class="w-10 h-10 rounded-xl object-cover bg-black/60 p-0.5 transition-colors duration-300" :class="p.speaking ? 'border-2 border-emerald-400 shadow-[0_0_15px_rgba(52,211,153,0.6)]' : 'border border-white/10'">
                <div class="flex-1 min-w-0">
                  <h4 class="font-bold text-sm truncate transition-colors duration-300" :class="p.speaking ? 'text-emerald-400' : 'text-white'">{{ p.name }}</h4>
                  <div class="flex items-center gap-1 mt-1.5 h-3">
                    <template v-if="p.speaking">
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.8s_ease-in-out_infinite]" style="height: 80%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.6s_ease-in-out_infinite_0.1s]" style="height: 100%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.9s_ease-in-out_infinite_0.2s]" style="height: 60%;"></span>
                      <span class="w-1 bg-emerald-400 rounded-full animate-[bounce_0.7s_ease-in-out_infinite_0.3s]" style="height: 90%;"></span>
                    </template>
                    <template v-else>
                      <span class="w-6 h-0.5 bg-white/20 rounded-full"></span>
                    </template>
                  </div>
                </div>
                <div class="flex gap-1.5 shrink-0 bg-black/40 px-2 py-1.5 rounded-lg border border-white/5">
                  <MicOff v-if="p.muted" class="w-3.5 h-3.5 text-red-400" />
                  <Headphones v-if="p.deafened" class="w-3.5 h-3.5 text-amber-400" />
                  <Mic v-if="!p.muted && !p.deafened" class="w-3.5 h-3.5 text-white/20" :class="p.speaking ? 'text-emerald-400' : ''" />
                </div>
              </div>
            </div>
          </div>

          <div class="flex justify-end pt-3">
            <button @click="leaveVoiceChannel" class="kip-btn-danger px-6 py-2.5">
              <PhoneOff class="w-4 h-4" /> Disconnect
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-6">
      <div class="kip-card p-8 relative overflow-hidden flex flex-col group hover:border-indigo-500/30 transition-colors duration-500 stagger-2">
        <div class="absolute -right-20 -top-20 w-64 h-64 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-indigo-500/20 transition-all duration-500"></div>

        <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
          <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20"><Activity class="text-indigo-400 w-5 h-5" /></div>
          {{ t('Server Ping') }}
        </h3>

        <div class="flex gap-3 mb-6 relative z-10">
          <input v-model="netIp" @keyup.enter="pingServer" type="text" placeholder="mc.hypixel.net" class="kip-input">
          <button @click="pingServer" :disabled="isPinging" class="kip-btn-primary px-6 min-w-[120px]">
            <Loader v-if="isPinging" class="w-5 h-5 animate-spin" />
            <span v-else>{{ t('Ping') }}</span>
          </button>
        </div>

        <div v-if="pingResult" class="bg-black/40 p-5 rounded-2xl border border-white/5 mt-auto relative z-10 shadow-inner">
          <div class="flex gap-5 items-center">
            <img :src="pingResult.icon || fallbackServerIcon" class="w-16 h-16 rounded-xl bg-black/60 p-1 object-contain border border-white/5 shadow-lg">
            <div class="flex-1 min-w-0">
              <div class="flex justify-between items-center mb-2">
                <span class="font-bold text-emerald-400 flex items-center gap-2 text-sm uppercase tracking-wider">
                  <span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse"></span> {{ t('Online') }}
                </span>
                <span class="text-xs font-mono px-2 py-1 rounded-lg border" :class="getPingColorClass(pingResult.ping || 0)">
                  {{ pingResult.ping ?? 0 }} ms
                </span>
              </div>
              <p class="text-xs text-white/60 truncate mb-3" v-html="sanitizeHTML(pingResult.motd || '')"></p>

              <div class="w-full">
                <div class="flex justify-between text-[10px] font-bold text-white/50 uppercase tracking-widest mb-1.5">
                  <span>{{ t('Players') }}</span>
                  <span>{{ pingResult.players?.split('/')[0] || 0 }} / {{ pingResult.players?.split('/')[1] || 0 }}</span>
                </div>
                <div class="w-full h-1.5 bg-black/60 border border-white/5 rounded-full overflow-hidden shadow-inner">
                  <div class="h-full bg-indigo-500 rounded-full shadow-[0_0_10px_rgba(99,102,241,0.8)]" :style="{ width: calculatePlayerPercentage(pingResult.players || '0/0') + '%' }"></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="kip-card p-0 relative overflow-hidden flex flex-col group hover:border-purple-500/30 transition-colors duration-500 min-h-[350px] stagger-2">
        <div class="absolute inset-0 bg-gradient-to-t from-purple-900/20 via-transparent to-transparent pointer-events-none z-0"></div>

        <div class="p-8 pb-0 relative z-20 flex justify-between items-start pointer-events-none">
          <h3 class="text-xl font-bold flex items-center gap-3">
            <div class="p-2 bg-purple-500/10 rounded-xl border border-purple-500/20 pointer-events-auto"><User class="text-purple-400 w-5 h-5" /></div>
            {{ t('Interactive 3D Skin') }}
          </h3>
        </div>

        <div class="px-8 mt-4 relative z-20 flex gap-3">
          <input v-model="netNick" @keyup.enter="loadSkin3D" type="text" placeholder="Notch" class="kip-input bg-black/60 backdrop-blur shadow-xl">
          <button @click="loadSkin3D" :disabled="isSkinLoading" class="kip-btn-primary bg-purple-500 hover:bg-purple-400 shadow-[0_0_15px_rgba(168,85,247,0.3)] px-6 min-w-[120px]">
            <Loader v-if="isSkinLoading" class="w-5 h-5 animate-spin" />
            <span v-else>{{ t('Render') }}</span>
          </button>
        </div>

        <div ref="skinContainer" class="absolute inset-0 flex justify-center items-center cursor-move z-10 pt-20">
          <div v-show="!hasSkinLoaded" class="text-white/30 flex flex-col items-center gap-3 absolute pointer-events-none">
            <Box class="w-12 h-12 opacity-30" />
            <span class="text-sm font-medium tracking-wide">{{ t('Enter nickname to render') }}</span>
          </div>
          <canvas ref="skinCanvas" class="drop-shadow-[0_20px_30px_rgba(0,0,0,0.8)] transition-opacity duration-500" :class="hasSkinLoaded ? 'opacity-100' : 'opacity-0'"></canvas>
        </div>

        <div v-if="hasSkinLoaded" class="absolute bottom-6 left-1/2 -translate-x-1/2 z-30 flex gap-2 bg-black/60 backdrop-blur-md p-1.5 rounded-2xl border border-white/10 shadow-2xl">
          <button @click="setAnimation('idle')" class="px-4 py-1.5 rounded-xl text-xs font-bold transition" :class="skinAnim === 'idle' ? 'bg-purple-500 text-white shadow-[0_0_10px_rgba(168,85,247,0.5)]' : 'text-white/50 hover:text-white hover:bg-white/5'">Idle</button>
          <button @click="setAnimation('walk')" class="px-4 py-1.5 rounded-xl text-xs font-bold transition" :class="skinAnim === 'walk' ? 'bg-purple-500 text-white shadow-[0_0_10px_rgba(168,85,247,0.5)]' : 'text-white/50 hover:text-white hover:bg-white/5'">Walk</button>
          <button @click="setAnimation('run')" class="px-4 py-1.5 rounded-xl text-xs font-bold transition" :class="skinAnim === 'run' ? 'bg-purple-500 text-white shadow-[0_0_10px_rgba(168,85,247,0.5)]' : 'text-white/50 hover:text-white hover:bg-white/5'">Run</button>
        </div>
      </div>

      <div class="kip-card p-8 relative overflow-hidden hover:border-amber-500/30 transition-colors duration-500 group col-span-2 stagger-3">
        <div class="absolute -right-20 -bottom-20 w-64 h-64 bg-amber-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-amber-500/20 transition-all duration-500"></div>

        <div class="flex justify-between items-start mb-6 relative z-10">
          <div>
            <h3 class="text-xl font-bold mb-1 flex items-center gap-3">
              <div class="p-2 bg-amber-500/10 rounded-xl border border-amber-500/20"><Radio class="text-amber-400 w-5 h-5" /></div>
              {{ t('LAN Tunnel') }}
            </h3>
            <p class="text-white/50 text-sm">{{ t('Play with friends over internet without Hamachi.') }}</p>
          </div>
        </div>
        <div class="flex gap-4 relative z-10 mt-auto">
          <input v-model="tunnelPort" type="text" :placeholder="t('Local Port (e.g. 25565)')" class="kip-input font-mono">
          <button @click="startTunnel" class="kip-btn-primary px-8 bg-amber-500 hover:bg-amber-400 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]">
            <Play class="w-4 h-4 fill-current" /> {{ t('Start Tunnel') }}
          </button>
          <button @click="stopTunnel" class="kip-btn-danger px-8 bg-red-500/10 hover:bg-red-500/20 text-red-400 hover:text-red-300 border border-red-500/20 shadow-none">
            <Square class="w-4 h-4 fill-current" /> {{ t('Stop') }}
          </button>
        </div>
      </div>

      <div class="kip-card p-8 relative overflow-hidden hover:border-blue-500/30 transition-colors duration-500 group col-span-2 stagger-3">
        <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-blue-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-blue-500/20 transition-all duration-500"></div>

        <div class="flex justify-between items-start mb-6 relative z-10">
          <div>
            <h3 class="text-xl font-bold mb-1 flex items-center gap-3">
              <div class="p-2 bg-blue-500/10 rounded-xl border border-blue-500/20"><ServerCog class="text-blue-400 w-5 h-5" /></div>
              {{ t('Pterodactyl Panel') }}
            </h3>
            <p class="text-white/50 text-sm">{{ t('Manage your remote server directly from the launcher.') }}</p>
          </div>
        </div>

        <div class="flex gap-3 mb-5 relative z-10">
          <input v-model="ptero.url" type="text" :placeholder="t('Panel URL (e.g. https://panel.example.com)')" class="kip-input flex-1">
          <input v-model="ptero.key" type="password" :placeholder="t('Client API Key')" class="kip-input w-1/3 font-mono">
          <button @click="pteroConnect" :disabled="ptero.loading" class="kip-btn-primary bg-blue-500 hover:bg-blue-400 px-6 min-w-[120px] shadow-[0_0_15px_rgba(59,130,246,0.3)]">
            <Loader v-if="ptero.loading" class="w-5 h-5 animate-spin" />
            <span v-else class="flex items-center gap-2"><LinkIcon class="w-4 h-4" /> {{ t('Connect') }}</span>
          </button>
        </div>

        <div v-if="ptero.servers.length > 0" class="mb-4 relative custom-dropdown z-20">
          <select v-model="ptero.serverId" @change="updateSelectedPteroServer" class="kip-input py-2.5 text-xs appearance-none cursor-pointer border-blue-500/30">
            <option v-for="s in ptero.servers" :key="s.id" :value="s.id">{{ s.name }} ({{ s.state }})</option>
          </select>
        </div>

        <div v-if="ptero.serverId" class="flex items-center justify-between bg-black/40 p-4 px-6 rounded-xl border border-white/5 relative z-10 shadow-inner">
          <div class="flex items-center gap-3">
            <span class="w-2.5 h-2.5 rounded-full" :class="ptero.status === 'running' ? 'bg-emerald-400 animate-pulse shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-amber-400'"></span>
            <span class="text-xs font-bold uppercase tracking-widest text-white/70">{{ ptero.status }}</span>
          </div>
          <div class="flex gap-2">
            <button @click="pteroAction('start')" class="kip-btn-ghost px-5 py-2 text-sm text-emerald-400 hover:text-white border-emerald-500/20 hover:bg-emerald-500/20">{{ t('Start') }}</button>
            <button @click="pteroAction('restart')" class="kip-btn-ghost px-5 py-2 text-sm text-amber-400 hover:text-white border-amber-500/20 hover:bg-amber-500/20">{{ t('Restart') }}</button>
            <button @click="pteroAction('kill')" class="kip-btn-ghost px-5 py-2 text-sm text-red-400 hover:text-white border-red-500/20 hover:bg-red-500/20">{{ t('Kill') }}</button>
          </div>
        </div>
      </div>

      <div class="kip-card p-8 relative overflow-hidden border-cyan-500/20 group col-span-2 stagger-3">
        <div class="absolute -right-20 -top-20 w-96 h-96 bg-cyan-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-cyan-500/20 transition-all duration-700"></div>

        <div class="flex justify-between items-start mb-6 relative z-10">
          <div>
            <h3 class="text-xl font-bold mb-1 flex items-center gap-3">
              <div class="p-2 bg-cyan-500/10 rounded-xl border border-cyan-500/20"><Container class="text-cyan-400 w-5 h-5" /></div>
              {{ t('1-Click Docker Server') }}
            </h3>
            <p class="text-white/50 text-sm">{{ t('Deploy isolated local servers via Docker.') }}</p>
          </div>
        </div>

        <div class="flex gap-4 relative z-10">
          <div class="relative w-1/4 custom-dropdown">
            <div @click="dockerDropdownOpen = !dockerDropdownOpen" class="bg-black/40 border rounded-xl px-5 py-3.5 flex justify-between items-center cursor-pointer transition hover:bg-black/60" :class="dockerDropdownOpen ? 'border-cyan-500 shadow-[0_0_15px_rgba(6,182,212,0.2)]' : 'border-white/10'">
              <span class="font-bold text-sm uppercase tracking-wider">{{ dockerCore }}</span>
              <ChevronDown class="w-4 h-4 text-white/50 transition-transform" :class="{'rotate-180': dockerDropdownOpen}" />
            </div>
            <transition name="fade">
              <div v-if="dockerDropdownOpen" class="absolute top-full left-0 w-full mt-2 bg-[#121214] border border-white/10 rounded-xl shadow-2xl overflow-hidden py-2 z-50">
                <div v-for="c in (['paper', 'fabric', 'forge', 'vanilla'] as const)" :key="c" @click="dockerCore = c; dockerDropdownOpen = false" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition font-bold text-sm uppercase tracking-wider" :class="dockerCore === c ? 'text-cyan-400' : 'text-white/70'">
                  {{ c }}
                </div>
              </div>
            </transition>
          </div>

          <input v-model="dockerVer" type="text" :placeholder="t('Ver (e.g. 1.20.4)')" class="kip-input w-1/4">
          <input v-model="dockerPort" type="text" :placeholder="t('Port')" class="kip-input w-1/4 font-mono">

          <button @click="deployDocker" :disabled="isDockerDeploying" class="kip-btn-primary w-1/4 bg-cyan-500 hover:bg-cyan-400 text-black shadow-[0_0_15px_rgba(6,182,212,0.4)]">
            <Loader v-if="isDockerDeploying" class="w-5 h-5 animate-spin" />
            <template v-else><Play class="w-5 h-5 fill-current" /> {{ t('Deploy') }}</template>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import {
  Activity,
  User,
  Box,
  Radio,
  Play,
  Square,
  ServerCog,
  Link as LinkIcon,
  Container,
  ChevronDown,
  Loader,
  Mic,
  MicOff,
  Headphones,
  PhoneOff,
  Settings,
  Hash,
  Dices,
  PhoneCall,
} from 'lucide-vue-next'
import { SkinViewer, WalkingAnimation, RunningAnimation, IdleAnimation } from 'skinview3d'
import {
  state,
  t,
  showToast,
  getAvatarUrl,
  sanitizeHTML,
  voiceState,
  toggleMute,
  toggleDeafen,
  leaveVoiceChannel,
  joinVoiceChannel,
  toggleVoiceSettings,
  setAudioInput,
  setAudioOutput,
  startMicTest,
  stopMicTest,
} from '@/store'
import {
  bridge,
  invokeSafe,
  type ServerPingResultDto,
  type GenericActionResult,
} from '@/bridge'

interface PteroServer {
  id: string
  name: string
  state: string
}

interface PteroState {
  url: string
  key: string
  status: string
  serverId: string
  servers: PteroServer[]
  loading: boolean
}

type SkinAnimationType = 'idle' | 'walk' | 'run'
type DockerCoreType = 'paper' | 'fabric' | 'forge' | 'vanilla'

const netIp = ref<string>('')
const isPinging = ref<boolean>(false)
const pingResult = ref<ServerPingResultDto | null>(null)

const netNick = ref<string>('')
const isSkinLoading = ref<boolean>(false)
const hasSkinLoaded = ref<boolean>(false)
const skinContainer = ref<HTMLElement | null>(null)
const skinCanvas = ref<HTMLCanvasElement | null>(null)
const skinAnim = ref<SkinAnimationType>('idle')
let skinViewerInstance: SkinViewer | null = null
let resizeObserver: ResizeObserver | null = null

const tunnelPort = ref<string>('')
const connectChannel = ref<string>('')

const ptero = ref<PteroState>({
  url: '',
  key: '',
  status: '',
  serverId: '',
  servers: [],
  loading: false,
})
let pteroInterval: ReturnType<typeof setInterval> | null = null

const dockerDropdownOpen = ref<boolean>(false)
const dockerCore = ref<DockerCoreType>('paper')
const dockerVer = ref<string>('')
const dockerPort = ref<string>('')
const isDockerDeploying = ref<boolean>(false)

const fallbackServerIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>'

const getPingColorClass = (ping: number): string => {
  if (ping < 50) return 'border-emerald-500/30 text-emerald-400 bg-emerald-500/10'
  if (ping < 150) return 'border-amber-500/30 text-amber-400 bg-amber-500/10'
  return 'border-red-500/30 text-red-400 bg-red-500/10'
}

const calculatePlayerPercentage = (playersStr: string): number => {
  try {
    const [online, max] = playersStr.split('/').map(Number)
    if (!max || max === 0 || Number.isNaN(online) || Number.isNaN(max)) return 0
    return Math.min(Math.round((online / max) * 100), 100)
  } catch {
    return 0
  }
}

const generateRandomRoom = (): string => {
  return 'kip-' + Math.random().toString(36).substring(2, 6)
}

const handleJoinVoice = (): void => {
  if (!connectChannel.value) {
    connectChannel.value = generateRandomRoom()
  }
  joinVoiceChannel(connectChannel.value)
}

const toggleMicTest = (): void => {
  if (voiceState.isTestingMic) {
    stopMicTest()
  } else {
    startMicTest()
  }
}

const pingServer = async (): Promise<void> => {
  const cleanIp = netIp.value.trim()
  if (!cleanIp) return
  isPinging.value = true
  pingResult.value = null
  let res: ServerPingResultDto | null = null

  try {
    res = await invokeSafe<ServerPingResultDto>('ping_server', { ip: cleanIp })
  } catch {
    res = null
  }

  if (!res || !res.online) {
    try {
      const ipParts = cleanIp.split(':')
      const fetchHost = ipParts[0] || 'localhost'
      const fetchPort = ipParts.length > 1 ? ipParts[1] : '25565'
      const webRes = await fetch(`https://api.mcstatus.io/v2/status/java/${fetchHost}:${fetchPort}`)
      const data = await webRes.json()
      if (data && data.online) {
        let motdText = 'Minecraft Server'
        if (data.motd && typeof data.motd.clean === 'string') {
          motdText = data.motd.clean
        } else if (data.motd && typeof data.motd.raw === 'string') {
          motdText = data.motd.raw.replace(/§[0-9a-fk-or]/gi, '')
        }

        res = {
          online: true,
          players: `${data.players?.online || 0}/${data.players?.max || 0}`,
          ping: 45,
          motd: motdText,
          icon: data.icon || null,
        }
      }
    } catch {
      // Ignored
    }
  }

  if (res && res.online) {
    pingResult.value = res
  } else {
    showToast(t('Offline'), t('Could not connect to server.'), 'danger')
  }
  isPinging.value = false
}

const loadSkin3D = async (): Promise<void> => {
  const nickname = netNick.value.trim()
  if (!nickname || !skinContainer.value || !skinCanvas.value) return
  isSkinLoading.value = true

  const width = skinContainer.value.clientWidth || 350
  const height = skinContainer.value.clientHeight || 400
  const url = `https://mc-heads.net/skin/${encodeURIComponent(nickname)}`

  try {
    if (!skinViewerInstance) {
      skinViewerInstance = new SkinViewer({
        canvas: skinCanvas.value,
        width,
        height,
        skin: url,
      })
      skinViewerInstance.fov = 70
      skinViewerInstance.zoom = 0.9
      skinViewerInstance.autoRotate = true
      skinViewerInstance.autoRotateSpeed = 0.5

      if (skinViewerInstance.controls) {
        skinViewerInstance.controls.enableRotate = true
        skinViewerInstance.controls.enableZoom = true
        skinViewerInstance.controls.enablePan = false
      }
      skinViewerInstance.animation = new IdleAnimation()
      skinAnim.value = 'idle'
    } else {
      await skinViewerInstance.loadSkin(url)
    }
    hasSkinLoaded.value = true
  } catch {
    showToast(t('Skin Error'), t('Failed to load skin for this player.'), 'danger')
  } finally {
    isSkinLoading.value = false
  }
}

const setAnimation = (type: SkinAnimationType): void => {
  if (!skinViewerInstance) return
  skinAnim.value = type
  if (type === 'walk') {
    skinViewerInstance.animation = new WalkingAnimation()
  } else if (type === 'run') {
    skinViewerInstance.animation = new RunningAnimation()
  } else {
    skinViewerInstance.animation = new IdleAnimation()
  }
}

const startTunnel = (): void => {
  const port = tunnelPort.value.trim()
  if (!port) return
  showToast(t('Tunnel'), t('Starting tunnel...'), 'info')
  try {
    bridge.startTunnel(port)
  } catch {
    showToast(t('Tunnel Error'), t('Could not execute tunnel command.'), 'danger')
  }
}

const stopTunnel = (): void => {
  try {
    bridge.stopTunnel()
  } catch {
    // Ignored
  }
  showToast(t('Tunnel'), t('Tunnel stopped.'), 'info')
}

const pteroConnect = async (): Promise<void> => {
  const url = ptero.value.url.trim()
  const key = ptero.value.key.trim()
  if (!url || !key) return
  ptero.value.loading = true

  try {
    const res = await invokeSafe<{ success: boolean; status?: PteroServer[] }>('ptero_connect', {
      url,
      key,
    })
    if (res && res.success && res.status) {
      ptero.value.servers = res.status
      if (ptero.value.servers.length > 0 && ptero.value.servers[0]) {
        ptero.value.serverId = ptero.value.servers[0].id
        ptero.value.status = ptero.value.servers[0].state
      }
      showToast(t('Connected'), t('Panel linked successfully.'), 'success')
      startPteroPolling()
    } else {
      showToast(t('Error'), t('Could not connect to panel.'), 'danger')
    }
  } catch {
    showToast(t('Error'), t('Could not connect to panel.'), 'danger')
  } finally {
    ptero.value.loading = false
  }
}

const updateSelectedPteroServer = (): void => {
  const srv = ptero.value.servers.find((s) => s.id === ptero.value.serverId)
  if (srv) {
    ptero.value.status = srv.state
  }
}

const pteroAction = async (action: 'start' | 'restart' | 'kill'): Promise<void> => {
  if (!ptero.value.serverId) return
  try {
    const res = await invokeSafe<{ success: boolean }>('ptero_action', {
      action,
      serverId: ptero.value.serverId,
    })
    if (res && res.success) {
      showToast(t('Command Sent'), t('Action executed.'), 'success')
    } else {
      showToast(t('Error'), t('Action failed.'), 'danger')
    }
  } catch {
    showToast(t('Error'), t('Action failed.'), 'danger')
  }
}

const startPteroPolling = (): void => {
  stopPteroPolling()
  if (ptero.value.serverId && ptero.value.url && ptero.value.key) {
    pteroInterval = setInterval(async () => {
      if (ptero.value.loading) return
      try {
        const res = await invokeSafe<{ success: boolean; status?: PteroServer[] }>('ptero_connect', {
          url: ptero.value.url,
          key: ptero.value.key,
        })
        if (res && res.success && res.status) {
          ptero.value.servers = res.status
          updateSelectedPteroServer()
        }
      } catch {
        // Ignored in polling
      }
    }, 5000)
  }
}

const stopPteroPolling = (): void => {
  if (pteroInterval) {
    clearInterval(pteroInterval)
    pteroInterval = null
  }
}

const deployDocker = async (): Promise<void> => {
  const ver = dockerVer.value.trim()
  const port = dockerPort.value.trim()
  if (!ver || !port) return
  isDockerDeploying.value = true

  try {
    const res = await invokeSafe<GenericActionResult>('deploy_docker_server', {
      core: dockerCore.value,
      version: ver,
      port,
    })
    if (res && res.success) {
      showToast(t('Deployed'), res.msg, 'success')
    } else {
      showToast(t('Error'), res?.msg || t('Docker deploy failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Docker deploy failed.'), 'danger')
  } finally {
    setTimeout(() => {
      isDockerDeploying.value = false
    }, 2000)
  }
}

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    dockerDropdownOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', closeDropdowns)

  resizeObserver = new ResizeObserver(() => {
    if (skinViewerInstance && skinContainer.value) {
      const w = skinContainer.value.clientWidth || 350
      const h = skinContainer.value.clientHeight || 400
      skinViewerInstance.setSize(w, h)
    }
  })

  if (skinContainer.value) {
    resizeObserver.observe(skinContainer.value)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
  stopPteroPolling()

  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }

  if (skinViewerInstance) {
    skinViewerInstance.dispose()
    skinViewerInstance = null
  }
})
</script>