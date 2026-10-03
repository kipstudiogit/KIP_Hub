<template>
  <div class="absolute top-6 right-6 w-[420px] max-h-[calc(100%-3rem)] flex flex-col z-[100] kip-card shadow-[0_0_60px_rgba(0,0,0,0.8)] overflow-hidden border-white/10 select-none animate-in fade-in slide-in-from-right duration-300">
    <div class="p-5 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
      <div class="flex items-center gap-2.5">
        <div class="p-2 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
          <Zap class="w-4 h-4 fill-current" />
        </div>
        <div>
          <h2 class="text-base font-black uppercase tracking-wider text-white flex items-center gap-2">
            K.I.P. Nexus
          </h2>
          <span class="text-[9px] font-mono font-bold text-white/40 uppercase tracking-widest">Global Social Fabric</span>
        </div>
      </div>
      <button @click="state.isNexusOpen = false" class="p-2 text-white/40 hover:text-white hover:bg-white/5 rounded-xl transition">
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex border-b border-white/5 bg-black/40 shrink-0">
      <button @click="activeTab = 'social'" class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2" :class="activeTab === 'social' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'">
        <Users class="w-3.5 h-3.5" /> Social ({{ state.friends.length }})
      </button>
      <button @click="activeTab = 'party'" class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2" :class="activeTab === 'party' ? 'text-amber-400 border-amber-400 bg-amber-500/5' : 'text-white/40 border-transparent hover:text-white'">
        <Gamepad2 class="w-3.5 h-3.5" /> Party Hub
      </button>
      <button @click="activeTab = 'identity'" class="flex-1 py-3 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2" :class="activeTab === 'identity' ? 'text-emerald-400 border-emerald-400 bg-emerald-500/5' : 'text-white/40 border-transparent hover:text-white'">
        <ShieldCheck class="w-3.5 h-3.5" /> Identity
      </button>
    </div>

    <div v-show="activeTab === 'social'" class="flex-1 flex flex-col min-h-0 bg-black/20">
      <div class="p-3 shrink-0 flex gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 text-white/30 w-3.5 h-3.5" />
          <input v-model="friendSearchQuery" type="text" :placeholder="t('Filter contacts...')" class="kip-input pl-9 py-2 text-xs">
        </div>
        <button @click="showAddFriendModal = !showAddFriendModal" class="kip-btn-primary px-3 py-2 text-xs shrink-0" :title="t('Add friend')">
          <UserPlus class="w-3.5 h-3.5" />
        </button>
      </div>

      <div v-if="showAddFriendModal" class="px-3 pb-3 shrink-0 flex gap-2">
        <input v-model="newFriendName" @keyup.enter="submitAddFriend" type="text" :placeholder="t('Enter operator tag...')" class="kip-input py-2 text-xs">
        <button @click="submitAddFriend" :disabled="!newFriendName.trim()" class="kip-btn-ghost px-4 text-xs font-bold text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/10">Add</button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scroll px-3 pb-3 flex flex-col gap-2 min-h-0">
        <div v-if="isFriendsLoading" class="py-12 text-center">
          <Loader class="w-6 h-6 animate-spin mx-auto text-indigo-400" />
        </div>
        <div v-else-if="filteredFriends.length === 0" class="py-10 text-center text-white/30 text-xs border border-dashed border-white/10 rounded-2xl mx-1">
          <span>{{ friendSearchQuery ? 'No contacts matching filter.' : 'Social roster is empty. Add operators above.' }}</span>
        </div>
        <div v-else v-for="friend in filteredFriends" :key="friend.name" class="p-3 bg-black/40 border border-white/5 hover:border-white/15 rounded-2xl flex items-center justify-between group transition-all duration-200">
          <div class="flex items-center gap-3 min-w-0">
            <div class="relative shrink-0">
              <img :src="getAvatarUrl(friend.name)" class="w-9 h-9 rounded-xl bg-black/60 p-0.5 object-cover border border-white/10">
              <span class="absolute -bottom-1 -right-1 flex h-2.5 w-2.5">
                <span v-if="friend.status === 'online'" class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span class="relative inline-flex rounded-full h-2.5 w-2.5 border border-[#09090B]" :class="friend.status === 'online' ? 'bg-emerald-500' : 'bg-white/20'"></span>
              </span>
            </div>
            <div class="flex flex-col min-w-0">
              <div class="flex items-center gap-1.5">
                <h4 class="font-black text-white text-xs truncate">{{ friend.name }}</h4>
                <Star v-if="friend.is_favorite" class="w-3 h-3 fill-amber-400 text-amber-400 shrink-0" />
              </div>
              <span class="text-[9px] font-mono text-white/40 truncate">
                {{ friend.activity || (friend.status === 'online' ? 'Active' : 'Offline') }}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
            <button @click="directCallFriend(friend)" class="p-2 text-emerald-400/70 hover:text-emerald-300 hover:bg-emerald-500/10 rounded-lg transition" :title="t('Invite to active voice room')">
              <PhoneCall class="w-3.5 h-3.5" />
            </button>
            <button @click="dispatchPartyInvite(friend.name)" class="p-2 text-amber-400/70 hover:text-amber-300 hover:bg-amber-500/10 rounded-lg transition" :title="t('Dispatch party link')">
              <Share2 class="w-3.5 h-3.5" />
            </button>
            <button @click="handleRemoveFriend(friend.name)" class="p-2 text-red-400/60 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition" :title="t('Remove friend')">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-show="activeTab === 'party'" class="flex-1 p-5 flex flex-col gap-4 overflow-y-auto custom-scroll bg-black/20">
      <div class="bg-black/40 border border-white/5 rounded-2xl p-5 flex flex-col gap-3">
        <div class="flex justify-between items-start">
          <div>
            <span class="text-[9px] font-black uppercase tracking-widest text-amber-400">Mesh Party Lobby</span>
            <h3 class="text-lg font-black text-white mt-0.5">P2P Instance Sync</h3>
          </div>
          <span class="px-2 py-0.5 rounded text-[9px] font-mono uppercase bg-amber-500/10 text-amber-400 border border-amber-500/20">Active</span>
        </div>
        <p class="text-xs text-white/50 leading-relaxed">
          Broadcast your mod list, world state, and network tunnel to party members with single-packet synchronization.
        </p>
        <button @click="broadcastPartyLobby" class="kip-btn-primary py-3 bg-amber-500 hover:bg-amber-400 text-black font-black text-xs uppercase tracking-wider shadow-[0_0_20px_rgba(245,158,11,0.3)]">
          <Radio class="w-4 h-4 fill-current" /> Broadcast Modpack & Tunnel
        </button>
      </div>

      <div class="kip-card p-4 flex flex-col gap-2.5 bg-black/40 border-white/5">
        <span class="text-[9px] font-black uppercase tracking-widest text-white/40">Active Room Audio Link</span>
        <div class="flex items-center justify-between text-xs font-mono">
          <span class="text-white/60">Voice Matrix:</span>
          <span class="text-emerald-400 font-bold">{{ voiceState.isConnected ? voiceState.channelId : 'Not Connected' }}</span>
        </div>
        <div class="flex items-center justify-between text-xs font-mono">
          <span class="text-white/60">Connected Operators:</span>
          <span class="text-white font-bold">{{ voiceState.participants.length + (voiceState.isConnected ? 1 : 0) }}</span>
        </div>
      </div>
    </div>

    <div v-show="activeTab === 'identity'" class="flex-1 p-5 flex flex-col gap-5 overflow-y-auto custom-scroll bg-black/20">
      <div class="bg-black/40 rounded-2xl p-5 border border-white/5 flex flex-col items-center text-center">
        <div class="relative w-16 h-16 mb-3">
          <div class="absolute inset-0 rounded-2xl blur-md transition-all duration-300" :class="state.settings.has_ms_token ? 'bg-emerald-500/40' : 'bg-white/10'"></div>
          <img v-if="state.settings.ms_name" :src="getAvatarUrl(state.settings.ms_name)" class="w-full h-full rounded-2xl object-cover border-2 relative z-10 border-black shadow-lg">
          <div v-else class="w-full h-full rounded-2xl border-2 border-black bg-[#121214] flex items-center justify-center relative z-10">
            <UserX class="w-6 h-6 text-white/30" />
          </div>
        </div>

        <h3 class="text-lg font-black text-white">{{ state.settings.ms_name || t('Guest Operator') }}</h3>
        <span class="text-[9px] font-bold uppercase tracking-widest mt-0.5 mb-4" :class="state.settings.has_ms_token ? 'text-emerald-400' : 'text-amber-400'">
          {{ state.settings.has_ms_token ? 'Microsoft OAuth Verified' : 'Local Offline Mode' }}
        </span>

        <div v-if="msAuthCode" class="w-full bg-black/60 border border-emerald-500/30 p-4 rounded-xl mb-4 text-center shadow-inner">
          <p class="text-[9px] font-black text-white/50 uppercase tracking-widest mb-1.5">Authorization Code</p>
          <div class="text-3xl font-mono font-bold text-emerald-400 tracking-[0.2em] select-all cursor-pointer">{{ msAuthCode }}</div>
          <p class="text-[9px] text-emerald-400/60 uppercase tracking-widest mt-2 animate-pulse">Awaiting Microsoft Confirmation...</p>
        </div>

        <button v-else-if="!state.settings.has_ms_token" @click="handleMsAuth" :disabled="isMsAuthing" class="kip-btn-primary w-full py-2.5 text-xs font-black tracking-wider uppercase">
          <Loader v-if="isMsAuthing" class="w-3.5 h-3.5 animate-spin" />
          <span v-else>Link Xbox Live Account</span>
        </button>

        <button v-else @click="handleMsLogout" class="kip-btn-danger w-full py-2.5 text-xs font-black tracking-wider uppercase">
          Unlink Microsoft Account
        </button>
      </div>

      <div class="bg-black/40 rounded-2xl p-5 border border-white/5 flex flex-col items-center">
        <template v-if="!state.settings.has_kip_token">
          <div class="w-full flex bg-black/60 p-1 rounded-xl mb-4 border border-white/5">
            <button @click="kipAuthForm.isLogin = true" :class="kipAuthForm.isLogin ? 'bg-indigo-500/20 text-indigo-400 font-black' : 'text-white/40'" class="flex-1 py-1.5 rounded-lg text-[10px] uppercase tracking-wider transition">Sign In</button>
            <button @click="kipAuthForm.isLogin = false" :class="!kipAuthForm.isLogin ? 'bg-indigo-500/20 text-indigo-400 font-black' : 'text-white/40'" class="flex-1 py-1.5 rounded-lg text-[10px] uppercase tracking-wider transition">Sign Up</button>
          </div>

          <div class="w-full flex flex-col gap-2 mb-4">
            <input v-model="kipAuthForm.username" type="text" placeholder="K.I.P. ID Handle" class="kip-input py-2 text-xs">
            <input v-if="!kipAuthForm.isLogin" v-model="kipAuthForm.email" type="email" placeholder="Security Email" class="kip-input py-2 text-xs">
            <input v-model="kipAuthForm.password" type="password" placeholder="Passcode" class="kip-input py-2 text-xs">
          </div>

          <button @click="handleKipSubmit" :disabled="kipAuthForm.loading" class="kip-btn-primary w-full py-2.5 text-xs font-black tracking-wider uppercase">
            <Loader v-if="kipAuthForm.loading" class="w-3.5 h-3.5 animate-spin" />
            <span v-else>{{ kipAuthForm.isLogin ? 'Authenticate ID' : 'Register Operator' }}</span>
          </button>
        </template>

        <template v-else>
          <div class="w-full flex items-center justify-between p-2">
            <div class="flex items-center gap-3">
              <Hexagon class="w-8 h-8 text-indigo-400" />
              <div class="text-left">
                <h4 class="font-black text-white text-sm leading-tight">{{ state.settings.kip_username }}</h4>
                <span class="text-[9px] font-mono text-indigo-400">Cloud Sync Active</span>
              </div>
            </div>
            <button @click="handleKipLogout" class="p-2 text-red-400 hover:bg-red-500/10 rounded-lg transition" :title="t('Sign out')">
              <LogOut class="w-4 h-4" />
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
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
  Share2,
  Trash2,
  Loader,
  Radio,
  UserX,
  Hexagon,
  LogOut,
} from 'lucide-vue-next'
import {
  state,
  t,
  showToast,
  getAvatarUrl,
  voiceState,
  loadSettings,
  inviteToParty,
  joinVoiceChannel,
} from '@/store'
import { bridge, type FriendRecord, type GenericActionResult } from '@/bridge'

const activeTab = ref<'social' | 'party' | 'identity'>('social')
const friendSearchQuery = ref<string>('')
const showAddFriendModal = ref<boolean>(false)
const newFriendName = ref<string>('')
const isFriendsLoading = ref<boolean>(false)

const isMsAuthing = ref<boolean>(false)
const msAuthCode = ref<string>('')

const kipAuthForm = reactive({
  isLogin: true,
  username: '',
  email: '',
  password: '',
  loading: false,
})

const filteredFriends = computed<FriendRecord[]>(() => {
  const query = friendSearchQuery.value.trim().toLowerCase()
  if (!query) return state.friends
  return state.friends.filter((f) => f.name.toLowerCase().includes(query))
})

const fetchFriends = async (): Promise<void> => {
  isFriendsLoading.value = true
  try {
    const list = await bridge.getFriends()
    state.friends = list || []
  } catch {
    state.friends = []
  } finally {
    isFriendsLoading.value = false
  }
}

const submitAddFriend = async (): Promise<void> => {
  const clean = newFriendName.value.trim()
  if (!clean) return

  try {
    const res: GenericActionResult = await bridge.addFriend(clean)
    if (res.success) {
      showToast(t('Operator Linked'), res.msg, 'success')
      newFriendName.value = ''
      showAddFriendModal.value = false
      await fetchFriends()
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Failed to link friend.'), 'danger')
  }
}

const handleRemoveFriend = async (name: string): Promise<void> => {
  try {
    const res: GenericActionResult = await bridge.removeFriend(name)
    if (res.success) {
      showToast(t('Removed'), res.msg, 'success')
      await fetchFriends()
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch {
    showToast(t('Error'), t('Failed to remove friend.'), 'danger')
  }
}

const directCallFriend = (friend: FriendRecord): void => {
  if (voiceState.isConnected) {
    inviteToParty(friend.name)
  } else {
    joinVoiceChannel('nexus-' + friend.name.toLowerCase().slice(0, 8))
    showToast(t('Call Initialized'), `Calling room: nexus-${friend.name.toLowerCase().slice(0, 8)}`, 'info')
  }
}

const dispatchPartyInvite = (friendName: string): void => {
  inviteToParty(friendName)
}

const broadcastPartyLobby = (): void => {
  if (state.friends.length === 0) {
    showToast(t('Roster Empty'), t('Add friends to broadcast party invitations.'), 'danger')
    return
  }
  state.friends.forEach((f) => {
    inviteToParty(f.name)
  })
  showToast(t('Mesh Broadcast'), `Transmitted party beacon to ${state.friends.length} operators.`, 'success')
}

const handleMsAuth = async (): Promise<void> => {
  isMsAuthing.value = true
  msAuthCode.value = ''
  try {
    const res = await bridge.msAuthStart()
    if (!res) {
      showToast(t('Error'), t('Could not reach Microsoft authentication server.'), 'danger')
      isMsAuthing.value = false
      return
    }

    if (res.user_code) {
      msAuthCode.value = res.user_code
    }

    for (let i = 0; i < 30; i++) {
      const authSuccess = await bridge.msAuthPoll(res.device_code)
      if (authSuccess) {
        await loadSettings()
        showToast(t('OAuth Verified'), t('Xbox account linked successfully.'), 'success')
        break
      }
      await new Promise((r) => setTimeout(r, 5000))
    }
  } catch {
    showToast(t('Error'), t('Microsoft OAuth failed.'), 'danger')
  } finally {
    isMsAuthing.value = false
    msAuthCode.value = ''
  }
}

const handleMsLogout = async (): Promise<void> => {
  try {
    await bridge.msLogout()
    await loadSettings()
    showToast(t('Unlinked'), t('Microsoft session purged.'), 'info')
  } catch {
    showToast(t('Error'), t('Failed to purge Microsoft token.'), 'danger')
  }
}

const handleKipSubmit = async (): Promise<void> => {
  if (kipAuthForm.loading) return
  if (!kipAuthForm.username.trim() || !kipAuthForm.password.trim()) {
    showToast(t('Error'), t('Username and password are required.'), 'danger')
    return
  }

  kipAuthForm.loading = true
  try {
    let res: { success: boolean; token?: string; username?: string; msg?: string }
    if (kipAuthForm.isLogin) {
      res = await bridge.kipLogin(kipAuthForm.username.trim(), kipAuthForm.password)
    } else {
      res = await bridge.kipRegister(
        kipAuthForm.username.trim(),
        kipAuthForm.email.trim(),
        kipAuthForm.password
      )
    }

    if (res?.success) {
      showToast(t('Authenticated'), kipAuthForm.isLogin ? t('K.I.P. ID active.') : t('Account registered.'), 'success')
      await loadSettings()
      kipAuthForm.password = ''
    } else {
      showToast(t('Error'), res?.msg || t('Authentication failed.'), 'danger')
    }
  } catch {
    showToast(t('Error'), t('Backend communication failed.'), 'danger')
  } finally {
    kipAuthForm.loading = false
  }
}

const handleKipLogout = async (): Promise<void> => {
  try {
    await bridge.kipLogout()
    await loadSettings()
    showToast(t('Logged Out'), t('Disconnected from K.I.P. Cloud ID.'), 'info')
  } catch {
    showToast(t('Error'), t('Failed to disconnect.'), 'danger')
  }
}

onMounted(() => {
  fetchFriends()
})
</script>