<template>
  <div class="absolute top-6 right-6 w-[430px] max-h-[calc(100%-3rem)] flex flex-col z-[100] kip-card shadow-[0_0_60px_rgba(0,0,0,0.85)] overflow-hidden border-white/10 select-none animate-in fade-in slide-in-from-right duration-300">
    <!-- Header -->
    <header class="p-5 border-b border-white/5 flex justify-between items-center bg-black/70 shrink-0">
      <div class="flex items-center gap-2.5">
        <div class="p-2 rounded-xl bg-indigo-500/20 border border-indigo-500/30 text-indigo-400">
          <Zap class="w-4 h-4 fill-current stroke-[2.2]" />
        </div>
        <div>
          <h2 class="text-base font-black uppercase tracking-wider text-white flex items-center gap-2 leading-none">
            K.I.P. Nexus
          </h2>
          <span class="text-[8px] font-mono font-bold text-white/40 uppercase tracking-widest block mt-0.5">Distributed Social Fabric</span>
        </div>
      </div>
      <button @click="state.isNexusOpen = false" class="p-2 text-white/40 hover:text-white hover:bg-white/5 rounded-xl transition">
        <X class="w-4 h-4" />
      </button>
    </header>

    <!-- Navigation Tabs -->
    <nav class="flex border-b border-white/5 bg-black/40 shrink-0">
      <button
        @click="activeTab = 'social'"
        class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
        :class="activeTab === 'social' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'"
      >
        <Users class="w-3.5 h-3.5" /> Social ({{ friends.length }})
      </button>
      <button
        @click="activeTab = 'party'; fetchPartyState()"
        class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
        :class="activeTab === 'party' ? 'text-amber-400 border-amber-400 bg-amber-500/5' : 'text-white/40 border-transparent hover:text-white'"
      >
        <Gamepad2 class="w-3.5 h-3.5" /> Party Hub
      </button>
      <button
        @click="activeTab = 'identity'"
        class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
        :class="activeTab === 'identity' ? 'text-emerald-400 border-emerald-400 bg-emerald-500/5' : 'text-white/40 border-transparent hover:text-white'"
      >
        <ShieldCheck class="w-3.5 h-3.5" /> Identity
      </button>
    </nav>

    <!-- TAB 1: SOCIAL MATRIX -->
    <div v-show="activeTab === 'social'" class="flex-1 flex flex-col min-h-0 bg-black/20">
      <div class="p-3 shrink-0 flex gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 text-white/30 w-3.5 h-3.5" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('Filter operators...')"
            class="kip-input pl-9 py-2 text-xs font-mono"
          >
        </div>
        <button
          @click="isAddingFriend = !isAddingFriend"
          class="kip-btn-primary px-3.5 py-2 text-xs shrink-0"
          :title="t('Add operator to roster')"
        >
          <UserPlus class="w-3.5 h-3.5" />
        </button>
      </div>

      <!-- Add Friend Collapsible Form -->
      <div v-if="isAddingFriend" class="px-3 pb-3 shrink-0 flex gap-2">
        <input
          v-model="newFriendInput"
          @keyup.enter="addFriend"
          type="text"
          placeholder="Operator nickname (e.g. Steve_99)..."
          class="kip-input py-2 text-xs font-mono"
        >
        <button
          @click="addFriend"
          :disabled="!newFriendInput.trim()"
          class="kip-btn-ghost px-4 text-xs font-bold text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/10"
        >
          Link
        </button>
      </div>

      <!-- Note Editor Popover -->
      <div v-if="activeEditingFriend" class="mx-3 mb-3 p-3 bg-black/60 border border-indigo-500/40 rounded-xl flex flex-col gap-2 shadow-2xl">
        <div class="flex justify-between items-center text-[10px] font-mono font-bold text-indigo-300">
          <span>Note for {{ activeEditingFriend.name }}:</span>
          <button @click="activeEditingFriend = null" class="text-white/40 hover:text-white"><X class="w-3 h-3" /></button>
        </div>
        <input v-model="editingNoteInput" @keyup.enter="saveFriendNote" class="kip-input py-1.5 text-xs font-mono" placeholder="PvP teammate, modpack tester...">
        <button @click="saveFriendNote" class="kip-btn-primary py-1.5 text-xs font-bold uppercase tracking-wider">Save Note</button>
      </div>

      <!-- Friends Scrollable Roster -->
      <div class="flex-1 overflow-y-auto custom-scroll px-3 pb-3 flex flex-col gap-2 min-h-0">
        <div v-if="isLoadingFriends" class="py-12 text-center">
          <Loader class="w-6 h-6 animate-spin mx-auto text-indigo-400" />
        </div>

        <div v-else-if="filteredFriends.length === 0" class="py-12 text-center text-white/30 text-xs border border-dashed border-white/10 rounded-2xl mx-1 font-mono">
          <span>{{ searchQuery ? 'No operators matching search.' : 'Social roster is empty. Add operators above.' }}</span>
        </div>

        <article
          v-for="friend in filteredFriends"
          :key="friend.name"
          class="p-3 bg-black/50 border border-white/5 hover:border-white/20 rounded-2xl flex items-center justify-between group transition-all duration-200"
        >
          <div class="flex items-center gap-3 min-w-0">
            <div class="relative shrink-0">
              <img :src="friend.avatar" class="w-10 h-10 rounded-xl bg-black/60 p-0.5 object-cover border border-white/10">
              <span class="absolute -bottom-1 -right-1 flex h-2.5 w-2.5">
                <span v-if="friend.status !== 'offline'" class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span
                  class="relative inline-flex rounded-full h-2.5 w-2.5 border border-[#09090B]"
                  :class="friend.status === 'in-game' ? 'bg-purple-500' : friend.status === 'online' ? 'bg-emerald-500' : 'bg-white/20'"
                ></span>
              </span>
            </div>

            <div class="flex flex-col min-w-0">
              <div class="flex items-center gap-1.5">
                <h4 class="font-black text-white text-xs truncate">{{ friend.name }}</h4>
                <button @click="toggleFavorite(friend)" :title="friend.isFavorite ? 'Unfavorite' : 'Favorite'">
                  <Star class="w-3 h-3 transition-colors" :class="friend.isFavorite ? 'fill-amber-400 text-amber-400' : 'text-white/20 hover:text-white/60'" />
                </button>
              </div>

              <span class="text-[9px] font-mono text-white/40 truncate">
                {{ friend.activity || (friend.status !== 'offline' ? 'Active' : 'Offline') }}
              </span>

              <span v-if="friend.note" class="text-[8px] font-mono text-indigo-300/80 truncate italic">
                # {{ friend.note }}
              </span>
            </div>
          </div>

          <!-- Quick Action Buttons -->
          <div class="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click="callFriend(friend)"
              class="p-2 text-emerald-400/80 hover:text-emerald-300 hover:bg-emerald-500/10 rounded-lg transition"
              :title="t('Direct voice invite')"
            >
              <PhoneCall class="w-3.5 h-3.5" />
            </button>

            <button
              @click="startEditNote(friend)"
              class="p-2 text-indigo-400/80 hover:text-indigo-300 hover:bg-indigo-500/10 rounded-lg transition"
              title="Edit Note"
            >
              <Edit2 class="w-3.5 h-3.5" />
            </button>

            <button
              @click="removeFriend(friend.name)"
              class="p-2 text-rose-400/60 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition"
              :title="t('Purge operator')"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </article>
      </div>
    </div>

    <!-- TAB 2: PARTY MESH DECK -->
    <div v-show="activeTab === 'party'" class="flex-1 p-5 flex flex-col gap-4 overflow-y-auto custom-scroll bg-black/20">
      <div class="bg-black/50 border border-white/5 rounded-2xl p-5 flex flex-col gap-3">
        <div class="flex justify-between items-start">
          <div>
            <span class="text-[9px] font-black uppercase tracking-widest text-amber-400 font-mono">Mesh Party Hub</span>
            <h3 class="text-lg font-black text-white mt-0.5">Instance P2P Sync</h3>
          </div>
          <span
            class="px-2 py-0.5 rounded text-[9px] font-mono uppercase font-bold border"
            :class="partyState.active ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-white/5 text-white/40 border-white/10'"
          >
            {{ partyState.active ? 'Tunnel Live' : 'Mesh Standby' }}
          </span>
        </div>

        <p class="text-xs text-white/50 leading-relaxed font-mono">
          Broadcast synchronized Minecraft mods and encrypted reverse tunnel endpoint to party members in one transmission.
        </p>

        <div v-if="partyState.hostEndpoint" class="p-3 bg-black/60 rounded-xl border border-amber-500/30 font-mono text-xs flex justify-between items-center">
          <span class="text-white font-bold truncate select-all">{{ partyState.hostEndpoint }}</span>
          <span class="text-[9px] text-amber-400 uppercase font-black">Active IP</span>
        </div>

        <button
          @click="broadcastParty"
          :disabled="isPartyBroadcasting"
          class="kip-btn-primary py-3 bg-amber-500 hover:bg-amber-400 text-black font-black text-xs uppercase tracking-wider shadow-[0_0_20px_rgba(245,158,11,0.3)] flex items-center justify-center gap-2 cursor-pointer"
        >
          <Loader v-if="isPartyBroadcasting" class="w-4 h-4 animate-spin" />
          <Radio v-else class="w-4 h-4 fill-current" />
          <span>Broadcast Modpack & Tunnel</span>
        </button>
      </div>

      <div class="kip-card p-4 flex flex-col gap-2.5 bg-black/40 border-white/5">
        <span class="text-[9px] font-black uppercase tracking-widest text-white/40 font-mono">Real-time Audio Lobby Link</span>
        <div class="flex items-center justify-between text-xs font-mono">
          <span class="text-white/60">Voice Matrix:</span>
          <span class="text-emerald-400 font-bold">{{ voiceState.isConnected ? voiceState.channelId : 'Disconnected' }}</span>
        </div>
        <div class="flex items-center justify-between text-xs font-mono">
          <span class="text-white/60">Connected Operators:</span>
          <span class="text-white font-bold">{{ voiceState.participants.length + (voiceState.isConnected ? 1 : 0) }}</span>
        </div>
      </div>
    </div>

    <!-- TAB 3: IDENTITY MATRIX -->
    <div v-show="activeTab === 'identity'" class="flex-1 p-5 flex flex-col gap-4 overflow-y-auto custom-scroll bg-black/20">
      <!-- Microsoft Xbox Live Card -->
      <div class="bg-black/50 rounded-2xl p-5 border border-white/5 flex flex-col items-center text-center">
        <div class="relative w-16 h-16 mb-3">
          <div class="absolute inset-0 rounded-2xl blur-md transition-all duration-300" :class="state.settings.has_ms_token ? 'bg-emerald-500/40' : 'bg-white/10'"></div>
          <img v-if="state.settings.ms_name" :src="getAvatarUrl(state.settings.ms_name)" class="w-full h-full rounded-2xl object-cover border-2 relative z-10 border-black shadow-lg">
          <div v-else class="w-full h-full rounded-2xl border-2 border-black bg-[#121214] flex items-center justify-center relative z-10">
            <UserX class="w-6 h-6 text-white/30" />
          </div>
        </div>

        <h3 class="text-base font-black text-white leading-tight">{{ state.settings.ms_name || t('Guest Operator') }}</h3>
        <span class="text-[8px] font-bold uppercase tracking-widest mt-0.5 mb-4 font-mono" :class="state.settings.has_ms_token ? 'text-emerald-400' : 'text-amber-400'">
          {{ state.settings.has_ms_token ? 'Microsoft OAuth Verified' : 'Local Offline Mode' }}
        </span>

        <div v-if="msAuthCode" class="w-full bg-black/70 border border-emerald-500/30 p-4 rounded-xl mb-4 text-center shadow-inner">
          <p class="text-[8px] font-black text-white/50 uppercase tracking-widest mb-1.5 font-mono">Authorization User Code</p>
          <div class="text-2xl font-mono font-bold text-emerald-400 tracking-[0.2em] select-all cursor-pointer">{{ msAuthCode }}</div>
          <p class="text-[9px] text-emerald-400/60 uppercase tracking-widest mt-2 animate-pulse font-mono">Awaiting Xbox Live Confirmation...</p>
        </div>

        <button
          v-else-if="!state.settings.has_ms_token"
          @click="startMsAuth"
          :disabled="isMsAuthing"
          class="kip-btn-primary w-full py-2.5 text-xs font-black tracking-wider uppercase"
        >
          <Loader v-if="isMsAuthing" class="w-3.5 h-3.5 animate-spin" />
          <span v-else>Link Xbox Live Account</span>
        </button>

        <button
          v-else
          @click="logoutMs"
          class="kip-btn-danger w-full py-2.5 text-xs font-black tracking-wider uppercase"
        >
          Unlink Microsoft Session
        </button>
      </div>

      <!-- K.I.P. Cloud ID PBKDF2 Card -->
      <div class="bg-black/50 rounded-2xl p-5 border border-white/5 flex flex-col items-center">
        <template v-if="!state.settings.has_kip_token">
          <div class="w-full flex bg-black/60 p-1 rounded-xl mb-3 border border-white/5 font-mono">
            <button
              @click="kipAuthForm.isLogin = true"
              :class="kipAuthForm.isLogin ? 'bg-indigo-500/20 text-indigo-400 font-black' : 'text-white/40'"
              class="flex-1 py-1.5 rounded-lg text-[10px] uppercase tracking-wider transition"
            >
              Sign In
            </button>
            <button
              @click="kipAuthForm.isLogin = false"
              :class="!kipAuthForm.isLogin ? 'bg-indigo-500/20 text-indigo-400 font-black' : 'text-white/40'"
              class="flex-1 py-1.5 rounded-lg text-[10px] uppercase tracking-wider transition"
            >
              Sign Up
            </button>
          </div>

          <div class="w-full flex flex-col gap-2 mb-3">
            <input v-model="kipAuthForm.username" type="text" placeholder="K.I.P. ID Handle" class="kip-input py-2 text-xs font-mono">
            <input v-if="!kipAuthForm.isLogin" v-model="kipAuthForm.email" type="email" placeholder="Security Email (Optional)" class="kip-input py-2 text-xs font-mono">
            <input v-model="kipAuthForm.password" type="password" placeholder="Passcode (Min 6 chars)" class="kip-input py-2 text-xs font-mono">
          </div>

          <button
            @click="submitKipAuth"
            :disabled="kipAuthForm.loading"
            class="kip-btn-primary w-full py-2.5 text-xs font-black tracking-wider uppercase"
          >
            <Loader v-if="kipAuthForm.loading" class="w-3.5 h-3.5 animate-spin" />
            <span v-else>{{ kipAuthForm.isLogin ? 'Authenticate ID' : 'Register Operator' }}</span>
          </button>
        </template>

        <template v-else>
          <div class="w-full flex items-center justify-between p-2">
            <div class="flex items-center gap-3">
              <Hexagon class="w-8 h-8 text-indigo-400 stroke-[1.8]" />
              <div class="text-left">
                <h4 class="font-black text-white text-sm leading-tight">{{ state.settings.kip_username }}</h4>
                <span class="text-[9px] font-mono text-emerald-400 font-bold uppercase tracking-wider">Cloud Sync Verified</span>
              </div>
            </div>
            <button @click="logoutKip" class="p-2 text-rose-400 hover:bg-rose-500/10 rounded-lg transition" :title="t('Sign out')">
              <LogOut class="w-4 h-4" />
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import {
  Zap,
  X,
  Users,
  Gamepad2,
  ShieldCheck,
  Search,
  UserPlus,
  Star,
  PhoneCall,
  Trash2,
  Loader,
  Radio,
  UserX,
  Hexagon,
  LogOut,
  Edit2,
} from 'lucide-vue-next'
import { state, t, getAvatarUrl, voiceState } from '@/store'
import { useNexus } from '../composables/useNexus'

const {
  activeTab,
  searchQuery,
  isAddingFriend,
  newFriendInput,
  isLoadingFriends,
  friends,
  filteredFriends,
  activeEditingFriend,
  editingNoteInput,
  partyState,
  isPartyBroadcasting,
  isMsAuthing,
  msAuthCode,
  kipAuthForm,
  fetchFriends,
  addFriend,
  removeFriend,
  toggleFavorite,
  startEditNote,
  saveFriendNote,
  fetchPartyState,
  callFriend,
  broadcastParty,
  startMsAuth,
  logoutMs,
  submitKipAuth,
  logoutKip,
} = useNexus()

onMounted(() => {
  fetchFriends()
  fetchPartyState()
})
</script>