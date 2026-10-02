/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

interface Window {
  __TAURI__?: {
    core?: {
      invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>
    }
    event?: {
      listen: <T = unknown>(
        event: string,
        handler: (e: { payload: T }) => void
      ) => Promise<() => void>
    }
  }
  __TAURI_INTERNALS__?: {
    invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>
    listen: <T = unknown>(
      event: string,
      handler: (e: { payload: T }) => void
    ) => Promise<() => void>
  }
  updateDownloadProgress?: ((filename: string, p: number) => void) | null
}