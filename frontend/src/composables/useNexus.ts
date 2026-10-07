import { ref, reactive, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { state, loadSettings } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'
import { voiceState, joinVoiceChannel, inviteToParty } from './useVoice'
import type { FriendDto, FriendActionDto, PartyStateDto } from '../types/nexus'

export function useNexus() {
  const activeTab = ref<'social' | 'party' | 'identity'>('social')
  const searchQuery = ref<string>('')
  const isAddingFriend = ref<boolean>(false)
  const newFriendInput = ref<string>('')
  const isLoadingFriends = ref<boolean>(false)

  // Friends & Roster State
  const friends = ref<FriendDto[]>([])
  const activeEditingFriend = ref<FriendDto | null>(null)
  const editingNoteInput = ref<string>('')

  // Party State
  const partyState = ref<PartyStateDto>({
    active: false,
    roomId: 'kip-party',
    hostEndpoint: null,
    membersCount: 1,
    installedModsCount: 0,
  })
  const isPartyBroadcasting = ref<boolean>(false)

  // Microsoft OAuth State
  const isMsAuthing = ref<boolean>(false)
  const msAuthCode = ref<string>('')
  let pollCancelFlag = false

  // K.I.P. ID State
  const kipAuthForm = reactive({
    isLogin: true,
    username: '',
    email: '',
    password: '',
    loading: false,
  })

  const filteredFriends = computed<FriendDto[]>(() => {
    const q = searchQuery.value.trim().toLowerCase()
    let list = [...friends.value]
    if (q) {
      list = list.filter((f) => f.name.toLowerCase().includes(q) || f.note.toLowerCase().includes(q))
    }
    return list
  })

  async function fetchFriends(): Promise<void> {
    isLoadingFriends.value = true
    try {
      friends.value = await invoke<FriendDto[]>('nexus_get_friends')
    } catch {
      friends.value = []
    } finally {
      isLoadingFriends.value = false
    }
  }

  async function addFriend(): Promise<void> {
    const clean = newFriendInput.value.trim()
    if (!clean) return

    try {
      const res = await invoke<FriendActionDto>('nexus_add_friend', { name: clean })
      if (res.success) {
        showToast(t('Operator Linked'), res.msg, 'success')
        newFriendInput.value = ''
        isAddingFriend.value = false
        await fetchFriends()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function removeFriend(name: string): Promise<void> {
    try {
      const res = await invoke<FriendActionDto>('nexus_remove_friend', { name })
      if (res.success) {
        showToast(t('Removed'), res.msg, 'success')
        await fetchFriends()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch {
      showToast(t('Error'), t('Failed to purge friend.'), 'danger')
    }
  }

  async function toggleFavorite(friend: FriendDto): Promise<void> {
    try {
      const ok = await invoke<boolean>('nexus_toggle_favorite', { name: friend.name })
      if (ok) {
        friend.isFavorite = !friend.isFavorite
        await fetchFriends()
      }
    } catch {}
  }

  function startEditNote(friend: FriendDto): void {
    activeEditingFriend.value = friend
    editingNoteInput.value = friend.note || ''
  }

  async function saveFriendNote(): Promise<void> {
    if (!activeEditingFriend.value) return
    try {
      await invoke<boolean>('nexus_update_note', {
        name: activeEditingFriend.value.name,
        note: editingNoteInput.value.trim(),
      })
      activeEditingFriend.value.note = editingNoteInput.value.trim()
      activeEditingFriend.value = null
    } catch {}
  }

  async function fetchPartyState(): Promise<void> {
    try {
      partyState.value = await invoke<PartyStateDto>('nexus_get_party_state')
    } catch {}
  }

  function callFriend(friend: FriendDto): void {
    if (voiceState.isConnected) {
      inviteToParty(friend.name)
    } else {
      joinVoiceChannel('nexus-' + friend.name.toLowerCase().slice(0, 8))
      showToast(t('Call Initialized'), `Calling room: nexus-${friend.name.toLowerCase().slice(0, 8)}`, 'info')
    }
  }

  function broadcastParty(): void {
    if (friends.value.length === 0) {
      showToast(t('Roster Empty'), t('Add friends to broadcast party invitations.'), 'danger')
      return
    }
    isPartyBroadcasting.value = true
    friends.value.forEach((f) => {
      inviteToParty(f.name)
    })
    showToast(t('Mesh Broadcast'), `Transmitted party beacon to ${friends.value.length} operators.`, 'success')
    setTimeout(() => {
      isPartyBroadcasting.value = false
    }, 1200)
  }

  async function startMsAuth(): Promise<void> {
    isMsAuthing.value = true
    msAuthCode.value = ''
    pollCancelFlag = false

    try {
      const res = await invoke<{ verification_uri?: string; user_code?: string; device_code: string }>('ms_auth_start')
      if (res?.user_code) {
        msAuthCode.value = res.user_code
      }

      for (let i = 0; i < 30; i++) {
        if (pollCancelFlag) break
        const verified = await invoke<boolean>('ms_auth_poll', { deviceCode: res.device_code })
        if (verified) {
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

  async function logoutMs(): Promise<void> {
    pollCancelFlag = true
    try {
      await invoke('ms_logout')
      await loadSettings()
      showToast(t('Unlinked'), t('Microsoft session purged.'), 'info')
    } catch {
      showToast(t('Error'), t('Failed to purge Microsoft token.'), 'danger')
    }
  }

  async function submitKipAuth(): Promise<void> {
    if (kipAuthForm.loading) return
    if (!kipAuthForm.username.trim() || !kipAuthForm.password.trim()) {
      showToast(t('Error'), t('Username and password are required.'), 'danger')
      return
    }

    kipAuthForm.loading = true
    try {
      let res: { success: boolean; token?: string; username?: string; msg?: string }
      if (kipAuthForm.isLogin) {
        res = await invoke('kip_login', {
          username: kipAuthForm.username.trim(),
          password: kipAuthForm.password,
        })
      } else {
        res = await invoke('kip_register', {
          username: kipAuthForm.username.trim(),
          email: kipAuthForm.email.trim() || null,
          password: kipAuthForm.password,
        })
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

  async function logoutKip(): Promise<void> {
    try {
      await invoke('kip_logout')
      await loadSettings()
      showToast(t('Logged Out'), t('Disconnected from K.I.P. Cloud ID.'), 'info')
    } catch {
      showToast(t('Error'), t('Failed to disconnect.'), 'danger')
    }
  }

  return {
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
  }
}