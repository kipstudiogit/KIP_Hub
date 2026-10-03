<template>
  <div class="h-full flex flex-col overflow-hidden select-none relative">
    <div class="flex justify-between items-center mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Network & Identity') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">High-Speed Relay, Server Radar & 3D Identity Lab</p>
      </div>

      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button @click="activeSubTab = 'voice'" class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2" :class="activeSubTab === 'voice' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'text-white/40 hover:text-white'">
          <Mic class="w-3.5 h-3.5" /> Voice Studio
        </button>
        <button @click="activeSubTab = 'radar'" class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2" :class="activeSubTab === 'radar' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-[0_0_20px_rgba(99,102,241,0.2)]' : 'text-white/40 hover:text-white'">
          <Activity class="w-3.5 h-3.5" /> Radar & Tunnel
        </button>
        <button @click="activeSubTab = 'remote'" class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2" :class="activeSubTab === 'remote' ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30 shadow-[0_0_20px_rgba(245,158,11,0.2)]' : 'text-white/40 hover:text-white'">
          <ServerCog class="w-3.5 h-3.5" /> Cloud Decks
        </button>
        <button @click="activeSubTab = 'skin'" class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2" :class="activeSubTab === 'skin' ? 'bg-purple-500/20 text-purple-400 border border-purple-500/30 shadow-[0_0_20px_rgba(168,85,247,0.2)]' : 'text-white/40 hover:text-white'">
          <User class="w-3.5 h-3.5" /> 3D Skin Lab
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto custom-scroll pr-2 pb-6 min-h-0 z-10">
      <div v-show="activeSubTab === 'voice'" class="flex flex-col gap-6">
        <div class="kip-card p-8 relative overflow-hidden flex flex-col border" :class="voiceState.isConnected ? 'border-emerald-500/30 bg-emerald-950/10 shadow-[0_0_40px_rgba(16,185,129,0.15)]' : 'border-white/5'">
          <div class="absolute -right-20 -top-20 w-72 h-72 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="flex justify-between items-center mb-6 relative z-10">
            <div class="flex items-center gap-3">
              <div class="p-3 rounded-2xl border" :class="voiceState.isConnected ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-400' : 'bg-white/5 border-white/10 text-white/40'">
                <Mic class="w-6 h-6" :class="voiceState.isConnected ? 'animate-pulse' : ''" />
              </div>
              <div>
                <h3 class="text-xl font-black text-white uppercase tracking-wider">K.I.P. Connect Studio</h3>
                <span class="text-[9px] font-mono text-white/40 uppercase tracking-widest">Low-Latency W3C Mesh • Formant Noise Canceller Active</span>
              </div>
            </div>

            <div v-if="voiceState.isConnected" class="flex items-center gap-3">
              <div class="flex items-center gap-2 bg-black/60 border border-emerald-500/30 px-4 py-2 rounded-xl font-mono text-xs text-emerald-400 font-bold">
                <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                <span>ROOM: {{ voiceState.channelId }}</span>
              </div>
              <button @click="toggleVoiceSettings" class="kip-btn-ghost p-2.5 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/10">
                <Settings class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div v-if="!voiceState.isConnected" class="flex flex-col gap-5 max-w-2xl relative z-10">
            <p class="text-xs text-white/60 leading-relaxed font-medium">
              Join or create an encrypted voice room. Connect across the internet with any room key or directly over local LAN.
            </p>

            <div class="flex gap-2 p-1 bg-black/40 rounded-xl border border-white/10 w-fit">
              <button @click="networkMode = 'global'" class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2" :class="networkMode === 'global' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-white/40 hover:text-white'">
                <Globe class="w-3.5 h-3.5" /> Global Cloud (Internet)
              </button>
              <button @click="networkMode = 'lan'" class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2" :class="networkMode === 'lan' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'">
                <Radio class="w-3.5 h-3.5" /> Local Network (LAN)
              </button>
            </div>

            <div class="flex gap-3">
              <div class="relative flex-1">
                <Hash class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
                <input v-model="connectChannel" @keyup.enter="handleJoinVoice" type="text" :placeholder="networkMode === 'global' ? t('Enter room key (e.g. survival)') : t('Local Host or IP:Port (e.g. 192.168.1.5:8765)')" class="kip-input pl-11 pr-12 font-mono">
                <button @click="connectChannel = generateRandomRoom()" class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-white/40 hover:text-emerald-400 transition" :title="t('Generate key')">
                  <Dices class="w-4 h-4" />
                </button>
              </div>
              <button @click="handleJoinVoice" class="kip-btn-primary px-8 py-3 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(16,185,129,0.3)]">
                <PhoneCall class="w-4 h-4 fill-current" /> Connect
              </button>
            </div>
          </div>

          <div v-else class="flex flex-col gap-6 relative z-10">
            <transition name="fade">
              <div v-if="voiceState.showSettings" class="bg-black/60 border border-white/10 rounded-2xl p-6 flex flex-col gap-5 shadow-2xl">
                <div class="grid grid-cols-2 gap-6">
                  <div>
                    <label class="text-[10px] font-black text-white/40 uppercase tracking-widest mb-2 block">Audio Input Device</label>
                    <select v-model="voiceState.selectedInputId" @change="setAudioInput(voiceState.selectedInputId)" class="kip-input py-2.5 text-xs">
                      <option value="default">Default Recording Device</option>
                      <option v-for="d in voiceState.inputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Microphone ' + d.deviceId.substring(0,4) }}</option>
                    </select>
                  </div>
                  <div>
                    <label class="text-[10px] font-black text-white/40 uppercase tracking-widest mb-2 block">Audio Output Device</label>
                    <select v-model="voiceState.selectedOutputId" @change="setAudioOutput(voiceState.selectedOutputId)" class="kip-input py-2.5 text-xs">
                      <option value="default">Default Playback Device</option>
                      <option v-for="d in voiceState.outputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Speaker ' + d.deviceId.substring(0,4) }}</option>
                    </select>
                  </div>
                </div>

                <div class="pt-4 border-t border-white/5 flex items-center justify-between gap-5">
                  <button @click="toggleMicTest" class="px-5 py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition border" :class="voiceState.isTestingMic ? 'bg-red-500/10 text-red-400 border-red-500/30' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'">
                    {{ voiceState.isTestingMic ? 'Stop Test' : 'Test Loopback' }}
                  </button>
                  <div class="flex-1 h-2 bg-black/60 rounded-full overflow-hidden border border-white/5 relative">
                    <div class="absolute top-0 left-0 h-full bg-emerald-400 transition-all duration-75 shadow-[0_0_10px_rgba(52,211,153,0.8)]" :style="{ width: voiceState.testMicVolume + '%' }"></div>
                  </div>
                </div>
              </div>
            </transition>

            <div class="grid grid-cols-4 gap-4">
              <div class="bg-black/50 border rounded-2xl p-4 flex flex-col justify-between transition-all duration-300" :class="voiceState.localSpeaking ? 'border-emerald-500/60 bg-emerald-500/10 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'border-white/5'">
                <div class="flex items-center gap-3">
                  <img :src="getAvatarUrl(state.settings.ms_name || 'Guest')" class="w-10 h-10 rounded-xl object-cover border border-white/10">
                  <div class="flex-1 min-w-0">
                    <h4 class="font-black text-sm text-white truncate">{{ state.settings.ms_name || 'Guest' }} (You)</h4>
                    <span class="text-[9px] font-mono text-emerald-400 font-bold uppercase">{{ voiceState.localSpeaking ? 'Transmitting' : 'Idle' }}</span>
                  </div>
                </div>

                <div class="flex items-center justify-between mt-4 pt-3 border-t border-white/5">
                  <div class="flex gap-1.5">
                    <button @click="toggleMute" class="p-2 rounded-lg transition" :class="voiceState.isMuted ? 'bg-red-500/20 text-red-400' : 'bg-white/5 text-white/60 hover:text-white'">
                      <MicOff v-if="voiceState.isMuted" class="w-3.5 h-3.5" />
                      <Mic v-else class="w-3.5 h-3.5" />
                    </button>
                    <button @click="toggleDeafen" class="p-2 rounded-lg transition" :class="voiceState.isDeafened ? 'bg-red-500/20 text-red-400' : 'bg-white/5 text-white/60 hover:text-white'">
                      <Headphones class="w-3.5 h-3.5" />
                    </button>
                  </div>
                  <span class="text-[9px] font-mono text-white/30 uppercase">Local Peer</span>
                </div>
              </div>

              <div v-for="p in voiceState.participants" :key="p.id" class="bg-black/50 border rounded-2xl p-4 flex flex-col justify-between transition-all duration-300" :class="p.speaking ? 'border-emerald-500/60 bg-emerald-500/10 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'border-white/5'">
                <div class="flex items-center gap-3">
                  <img :src="getAvatarUrl(p.name)" class="w-10 h-10 rounded-xl object-cover border border-white/10">
                  <div class="flex-1 min-w-0">
                    <h4 class="font-black text-sm text-white truncate">{{ p.name }}</h4>
                    <span class="text-[9px] font-mono font-bold uppercase" :class="p.speaking ? 'text-emerald-400' : 'text-white/40'">{{ p.speaking ? 'Speaking' : 'Listening' }}</span>
                  </div>
                </div>

                <div class="flex items-center justify-between mt-4 pt-3 border-t border-white/5">
                  <div class="flex gap-1.5">
                    <MicOff v-if="p.muted" class="w-3.5 h-3.5 text-red-400" />
                    <Headphones v-if="p.deafened" class="w-3.5 h-3.5 text-amber-400" />
                    <span v-if="!p.muted && !p.deafened" class="w-2 h-2 rounded-full bg-emerald-400"></span>
                  </div>
                  <span class="text-[9px] font-mono text-white/30 uppercase">Remote</span>
                </div>
              </div>
            </div>

            <div class="flex justify-end pt-2">
              <button @click="leaveVoiceChannel" class="kip-btn-danger px-6 py-2.5 text-xs uppercase font-black tracking-wider">
                <PhoneOff class="w-4 h-4" /> Disconnect Studio
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-show="activeSubTab === 'radar'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-indigo-500/30 transition">
          <div class="flex justify-between items-start mb-6">
            <div>
              <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
                <Activity class="w-5 h-5 text-indigo-400" /> Server Radar
              </h3>
              <p class="text-xs text-white/40 mt-1">Real-time latency check and MOTD diagnostic</p>
            </div>
          </div>

          <div class="flex gap-3 mb-6">
            <input v-model="netIp" @keyup.enter="pingServer" type="text" placeholder="mc.hypixel.net" class="kip-input font-mono">
            <button @click="pingServer" :disabled="isPinging" class="kip-btn-primary px-6 min-w-[110px] text-xs font-black uppercase">
              <Loader v-if="isPinging" class="w-4 h-4 animate-spin" />
              <span v-else>Inspect</span>
            </button>
          </div>

          <div v-if="pingResult" class="bg-black/50 p-5 rounded-2xl border border-white/5 flex flex-col gap-4 mt-auto">
            <div class="flex items-center gap-4">
              <img :src="pingResult.icon || fallbackServerIcon" class="w-14 h-14 rounded-xl bg-black/60 p-1 object-contain border border-white/10">
              <div class="flex-1 min-w-0">
                <div class="flex justify-between items-center mb-1">
                  <span class="text-xs font-black uppercase tracking-wider text-emerald-400 flex items-center gap-1.5">
                    <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span> Online
                  </span>
                  <span class="text-xs font-mono font-bold px-2 py-0.5 rounded border" :class="getPingColorClass(pingResult.ping || 0)">
                    {{ pingResult.ping ?? 0 }} ms
                  </span>
                </div>
                <p class="text-xs text-white/60 truncate font-mono" v-html="sanitizeHTML(pingResult.motd || '')"></p>
              </div>
            </div>

            <div>
              <div class="flex justify-between text-[10px] font-mono uppercase text-white/40 mb-1.5 font-bold">
                <span>Players Online</span>
                <span>{{ pingResult.players || '0/0' }}</span>
              </div>
              <div class="w-full h-1.5 bg-black/60 rounded-full overflow-hidden border border-white/5">
                <div class="h-full bg-indigo-500 rounded-full shadow-[0_0_10px_rgba(99,102,241,0.8)]" :style="{ width: calculatePlayerPercentage(pingResult.players || '0/0') + '%' }"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-amber-500/30 transition">
          <div class="flex justify-between items-start mb-6">
            <div>
              <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
                <Radio class="w-5 h-5 text-amber-400" /> LAN Reverse Tunnel
              </h3>
              <p class="text-xs text-white/40 mt-1">Expose singleplayer LAN game to friends via encrypted TCP bridge</p>
            </div>
          </div>

          <div class="flex flex-col gap-4">
            <div class="flex gap-3">
              <input v-model="tunnelPort" type="text" placeholder="25565" class="kip-input font-mono w-1/3 text-center">
              <button @click="startTunnel" class="kip-btn-primary flex-1 bg-amber-500 hover:bg-amber-400 text-black font-black uppercase text-xs shadow-[0_0_20px_rgba(245,158,11,0.3)]">
                <Play class="w-4 h-4 fill-current" /> Open Tunnel
              </button>
              <button @click="stopTunnel" class="kip-btn-danger px-5 bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white border border-red-500/30">
                <Square class="w-4 h-4 fill-current" />
              </button>
            </div>

            <div v-if="activeTunnelEndpoint" class="bg-black/60 p-5 rounded-2xl border border-amber-500/30 flex flex-col gap-3 shadow-inner">
              <span class="text-[9px] font-mono uppercase text-amber-400 font-bold tracking-widest">Live Tunnel Public Address:</span>
              <div class="flex items-center justify-between bg-black/80 px-4 py-3 rounded-xl border border-white/10">
                <span class="font-mono text-sm text-white font-bold select-all">{{ activeTunnelEndpoint }}</span>
                <button @click="copyTunnelAddress" class="kip-btn-ghost px-3 py-1.5 text-xs text-amber-400 border-amber-500/30 hover:bg-amber-500/10">
                  <Copy class="w-3.5 h-3.5" /> Copy IP
                </button>
              </div>
              <p class="text-[10px] text-white/50 leading-relaxed font-mono">
                Give this exact address to your friends. They can paste it directly into Minecraft Multiplayer Direct Connect!
              </p>
            </div>
          </div>
        </div>
      </div>

      <div v-show="activeSubTab === 'remote'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-blue-500/30 transition">
          <h3 class="text-xl font-black uppercase text-white flex items-center gap-2 mb-2">
            <ServerCog class="w-5 h-5 text-blue-400" /> Pterodactyl Panel Link
          </h3>
          <p class="text-xs text-white/40 mb-6">Manage remote dedicated gaming servers via official client API</p>

          <div class="flex flex-col gap-4">
            <input v-model="ptero.url" type="text" placeholder="https://panel.example.com" class="kip-input">
            <div class="flex gap-3">
              <input v-model="ptero.key" type="password" placeholder="Client API Key" class="kip-input font-mono flex-1">
              <button @click="pteroConnect" :disabled="ptero.loading" class="kip-btn-primary px-6 bg-blue-500 hover:bg-blue-400 text-white font-black uppercase text-xs shadow-[0_0_20px_rgba(59,130,246,0.3)]">
                <Loader v-if="ptero.loading" class="w-4 h-4 animate-spin" />
                <span v-else>Link</span>
              </button>
            </div>

            <div v-if="ptero.servers.length > 0" class="flex flex-col gap-3 mt-4 pt-4 border-t border-white/5">
              <select v-model="ptero.serverId" @change="updateSelectedPteroServer" class="kip-input text-xs">
                <option v-for="s in ptero.servers" :key="s.id" :value="s.id">{{ s.name }} ({{ s.state }})</option>
              </select>

              <div class="flex items-center justify-between bg-black/60 p-4 rounded-xl border border-white/5">
                <div class="flex items-center gap-2">
                  <span class="w-2.5 h-2.5 rounded-full" :class="ptero.status === 'running' ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'"></span>
                  <span class="text-xs font-mono font-bold uppercase text-white/80">{{ ptero.status }}</span>
                </div>
                <div class="flex gap-2">
                  <button @click="pteroAction('start')" class="kip-btn-ghost px-4 py-1.5 text-xs text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10">Start</button>
                  <button @click="pteroAction('restart')" class="kip-btn-ghost px-4 py-1.5 text-xs text-amber-400 border-amber-500/20 hover:bg-amber-500/10">Restart</button>
                  <button @click="pteroAction('kill')" class="kip-btn-ghost px-4 py-1.5 text-xs text-red-400 border-red-500/20 hover:bg-red-500/10">Kill</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-cyan-500/30 transition">
          <h3 class="text-xl font-black uppercase text-white flex items-center gap-2 mb-2">
            <Container class="w-5 h-5 text-cyan-400" /> 1-Click Docker Instance
          </h3>
          <p class="text-xs text-white/40 mb-6">Deploy isolated sandboxed containerized server on your local machine</p>

          <div class="flex flex-col gap-4">
            <div class="grid grid-cols-3 gap-3">
              <select v-model="dockerCore" class="kip-input text-xs font-bold uppercase">
                <option value="paper">PaperMC</option>
                <option value="fabric">Fabric</option>
                <option value="forge">Forge</option>
                <option value="vanilla">Vanilla</option>
              </select>
              <input v-model="dockerVer" type="text" placeholder="1.20.4" class="kip-input font-mono text-center text-xs">
              <input v-model="dockerPort" type="text" placeholder="25565" class="kip-input font-mono text-center text-xs">
            </div>

            <button @click="deployDocker" :disabled="isDockerDeploying" class="kip-btn-primary py-3.5 bg-cyan-500 hover:bg-cyan-400 text-black font-black uppercase text-xs shadow-[0_0_20px_rgba(6,182,212,0.3)]">
              <Loader v-if="isDockerDeploying" class="w-4 h-4 animate-spin" />
              <span v-else>Deploy Docker Stack</span>
            </button>
          </div>
        </div>
      </div>

      <div v-show="activeSubTab === 'skin'" class="kip-card p-8 flex flex-col border border-white/5 hover:border-purple-500/30 transition relative overflow-hidden min-h-[500px]">
        <div class="absolute -top-32 -right-32 w-96 h-96 bg-purple-500/10 blur-[120px] rounded-full pointer-events-none"></div>

        <div class="flex justify-between items-center mb-6 relative z-20">
          <div>
            <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
              <User class="w-5 h-5 text-purple-400" /> Holographic 3D Skin Lab
            </h3>
            <p class="text-xs text-white/40 mt-1">Real-time WebGL Minecraft player model renderer</p>
          </div>

          <div class="flex gap-3">
            <input v-model="netNick" @keyup.enter="loadSkin3D" type="text" placeholder="Notch" class="kip-input py-2 text-xs font-mono w-48">
            <button @click="loadSkin3D" :disabled="isSkinLoading" class="kip-btn-primary px-6 bg-purple-500 hover:bg-purple-400 text-white font-black text-xs uppercase shadow-[0_0_20px_rgba(168,85,247,0.3)]">
              <Loader v-if="isSkinLoading" class="w-4 h-4 animate-spin" />
              <span v-else>Project</span>
            </button>
          </div>
        </div>

        <div ref="skinContainer" class="flex-1 w-full flex justify-center items-center relative z-10 min-h-[350px]">
          <div v-show="!hasSkinLoaded" class="text-white/30 flex flex-col items-center gap-3">
            <Box class="w-12 h-12 opacity-30" />
            <span class="text-xs font-mono">Enter Minecraft nickname above to generate hologram</span>
          </div>
          <canvas ref="skinCanvas" class="drop-shadow-[0_20px_40px_rgba(0,0,0,0.9)] transition-opacity duration-500" :class="hasSkinLoaded ? 'opacity-100' : 'opacity-0'"></canvas>
        </div>

        <div v-if="hasSkinLoaded" class="flex justify-center gap-2 mt-4 relative z-20">
          <button @click="setAnimation('idle')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'idle' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Idle</button>
          <button @click="setAnimation('walk')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'walk' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Walk</button>
          <button @click="setAnimation('run')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'run' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Run</button>
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
  Container,
  Loader,
  Mic,
  MicOff,
  Headphones,
  PhoneOff,
  Settings,
  Hash,
  Dices,
  PhoneCall,
  Globe,
  Copy,
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

type SubTabKey = 'voice' | 'radar' | 'remote' | 'skin'
type SkinAnimationType = 'idle' | 'walk' | 'run'
type DockerCoreType = 'paper' | 'fabric' | 'forge' | 'vanilla'

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

const activeSubTab = ref<SubTabKey>('voice')
const networkMode = ref<'global' | 'lan'>('global')
const connectChannel = ref<string>('')

const netIp = ref<string>('')
const isPinging = ref<boolean>(false)
const pingResult = ref<ServerPingResultDto | null>(null)

const tunnelPort = ref<string>('25565')
const activeTunnelEndpoint = ref<string>('')

const ptero = ref<PteroState>({
  url: '',
  key: '',
  status: '',
  serverId: '',
  servers: [],
  loading: false,
})
let pteroInterval: ReturnType<typeof setInterval> | null = null

const dockerCore = ref<DockerCoreType>('paper')
const dockerVer = ref<string>('1.20.4')
const dockerPort = ref<string>('25565')
const isDockerDeploying = ref<boolean>(false)

const netNick = ref<string>('')
const isSkinLoading = ref<boolean>(false)
const hasSkinLoaded = ref<boolean>(false)
const skinContainer = ref<HTMLElement | null>(null)
const skinCanvas = ref<HTMLCanvasElement | null>(null)
const skinAnim = ref<SkinAnimationType>('idle')
let skinViewerInstance: SkinViewer | null = null
let resizeObserver: ResizeObserver | null = null

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
  const cleanInput = connectChannel.value.trim()
  if (networkMode.value === 'lan') {
    if (cleanInput.includes(':')) {
      const parts = cleanInput.split('/')
      const hostPart = parts[0]
      const roomPart = parts[1] || 'lan-room'
      joinVoiceChannel(roomPart, hostPart)
    } else {
      joinVoiceChannel(cleanInput, '127.0.0.1:8765')
    }
  } else {
    joinVoiceChannel(cleanInput, 'wss://kip-backend.noisyfutlor98.workers.dev/ws')
  }
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
    }
  }

  if (res && res.online) {
    pingResult.value = res
  } else {
    showToast(t('Offline'), t('Could not connect to server.'), 'danger')
  }
  isPinging.value = false
}

const startTunnel = (): void => {
  const port = tunnelPort.value.trim()
  if (!port) return
  showToast(t('Tunnel'), t('Starting reverse tunnel...'), 'info')
  try {
    bridge.startTunnel(port)
  } catch {
    showToast(t('Tunnel Error'), t('Could not execute tunnel command.'), 'danger')
  }
}

const stopTunnel = (): void => {
  try {
    bridge.stopTunnel()
    activeTunnelEndpoint.value = ''
  } catch {
  }
  showToast(t('Tunnel'), t('Tunnel stopped.'), 'info')
}

const copyTunnelAddress = async (): Promise<void> => {
  if (!activeTunnelEndpoint.value) return
  try {
    await navigator.clipboard.writeText(activeTunnelEndpoint.value)
    showToast(t('Copied'), 'Server address copied to clipboard!', 'success')
  } catch {
  }
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

onMounted(() => {
  if (window.__TAURI__?.event?.listen) {
    window.__TAURI__.event.listen<string>('updateTunnelStatus', (e) => {
      if (e.payload && !e.payload.includes('Error') && !e.payload.includes('NO_SSH')) {
        activeTunnelEndpoint.value = e.payload
      }
    })
  }

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