import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  ServerPingResultDto,
  PteroServerDto,
  NetworkActionResultDto,
  DockerCoreType,
} from '../types/network'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useNetworkManager() {
  const netIp = ref('')
  const isPinging = ref(false)
  const pingResult = ref<ServerPingResultDto | null>(null)
  const recentPings = ref<string[]>(['mc.hypixel.net', '2b2t.org'])

  // Tunnel state
  const tunnelPort = ref('25565')
  const activeTunnelEndpoint = ref('')
  let unlistenTunnel: UnlistenFn | null = null

  // Pterodactyl state
  const pteroUrl = ref('')
  const pteroKey = ref('')
  const pteroServers = ref<PteroServerDto[]>([])
  const selectedPteroServerId = ref('')
  const isPteroLoading = ref(false)
  let pteroInterval: ReturnType<typeof setInterval> | null = null

  // Docker state
  const dockerCore = ref<DockerCoreType>('paper')
  const dockerVer = ref('1.21.1')
  const dockerPort = ref('25565')
  const isDockerDeploying = ref(false)

  async function pingTargetServer(targetHost?: string): Promise<ServerPingResultDto | null> {
    const host = (targetHost || netIp.value).trim()
    if (!host) return null
    isPinging.value = true
    pingResult.value = null

    try {
      const res = await invoke<ServerPingResultDto>('ping_server', { ip: host })
      if (res && res.online) {
        pingResult.value = res
        if (!recentPings.value.includes(host)) {
          recentPings.value.unshift(host)
          if (recentPings.value.length > 5) recentPings.value.pop()
        }
      } else {
        showToast(t('Offline'), t('Could not connect to server.'), 'danger')
      }
      return res
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Ping Fault'), error.message || String(err), 'danger')
      return null
    } finally {
      isPinging.value = false
    }
  }

  function startReverseTunnel(): void {
    const port = tunnelPort.value.trim()
    if (!port) return
    showToast(t('Tunnel'), t('Starting reverse tunnel...'), 'info')
    invoke('start_tunnel', { port }).catch((err) => {
      showToast(t('Tunnel Fault'), String(err), 'danger')
    })
  }

  function stopReverseTunnel(): void {
    invoke('stop_tunnel').catch(() => {})
    activeTunnelEndpoint.value = ''
    showToast(t('Tunnel'), t('Tunnel stopped.'), 'info')
  }

  async function connectPterodactyl(): Promise<void> {
    const url = pteroUrl.value.trim()
    const key = pteroKey.value.trim()
    if (!url || !key) return
    isPteroLoading.value = true

    try {
      const servers = await invoke<PteroServerDto[]>('ptero_connect', { url, key })
      pteroServers.value = servers
      if (servers.length > 0 && servers[0]) {
        selectedPteroServerId.value = servers[0].id
      }
      showToast(t('Connected'), t('Panel linked successfully.'), 'success')
      startPteroPolling()
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Panel Fault'), error.message || String(err), 'danger')
    } finally {
      isPteroLoading.value = false
    }
  }

  async function sendPteroAction(action: 'start' | 'restart' | 'kill'): Promise<void> {
    if (!selectedPteroServerId.value) return
    try {
      const res = await invoke<NetworkActionResultDto>('ptero_action', {
        action,
        serverId: selectedPteroServerId.value,
      })
      if (res.success) {
        showToast(t('Command Dispatched'), res.msg, 'success')
      } else {
        showToast(t('Action Rejected'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  function startPteroPolling(): void {
    stopPteroPolling()
    if (pteroUrl.value && pteroKey.value) {
      pteroInterval = setInterval(async () => {
        try {
          pteroServers.value = await invoke<PteroServerDto[]>('ptero_connect', {
            url: pteroUrl.value,
            key: pteroKey.value,
          })
        } catch {}
      }, 5000)
    }
  }

  function stopPteroPolling(): void {
    if (pteroInterval) {
      clearInterval(pteroInterval)
      pteroInterval = null
    }
  }

  async function deployDocker(): Promise<void> {
    isDockerDeploying.value = true
    try {
      const res = await invoke<NetworkActionResultDto>('deploy_docker_server', {
        core: dockerCore.value,
        version: dockerVer.value.trim(),
        port: dockerPort.value.trim(),
      })
      if (res.success) {
        showToast(t('Deployed'), res.msg, 'success')
      } else {
        showToast(t('Deploy Notice'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Docker Fault'), error.message || String(err), 'danger')
    } finally {
      isDockerDeploying.value = false
    }
  }

  async function setupListeners(): Promise<void> {
    unlistenTunnel = await listen<string>('updateTunnelStatus', (e: Event<string>) => {
      if (e.payload && !e.payload.includes('Error') && !e.payload.includes('NO_SSH')) {
        activeTunnelEndpoint.value = e.payload
      }
    })
  }

  function cleanup(): void {
    if (unlistenTunnel) {
      unlistenTunnel()
      unlistenTunnel = null
    }
    stopPteroPolling()
  }

  return {
    netIp,
    isPinging,
    pingResult,
    recentPings,
    tunnelPort,
    activeTunnelEndpoint,
    pteroUrl,
    pteroKey,
    pteroServers,
    selectedPteroServerId,
    isPteroLoading,
    dockerCore,
    dockerVer,
    dockerPort,
    isDockerDeploying,
    pingTargetServer,
    startReverseTunnel,
    stopReverseTunnel,
    connectPterodactyl,
    sendPteroAction,
    deployDocker,
    setupListeners,
    cleanup,
  }
}