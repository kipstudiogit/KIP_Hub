import { ref } from 'vue'
import type { ToastItem } from '../types/app'

export const toasts = ref<ToastItem[]>([])
let toastIdCounter = 0

export function showToast(title: string, message: string, type: 'info' | 'success' | 'danger' = 'info'): void {
  const id = toastIdCounter++
  let icon = 'info'
  if (type === 'success') icon = 'check-circle'
  if (type === 'danger') icon = 'x-circle'

  toasts.value.push({ id, title, message, type, icon })
  if (toasts.value.length > 5) toasts.value.shift()

  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }, 4000)
}