export function sanitizeHTML(str: string): string {
  const temp = document.createElement('div')
  temp.textContent = str
  return temp.innerHTML
}

export function getAvatarUrl(name: string): string {
  if (!name) return ''
  return `https://mc-heads.net/avatar/${encodeURIComponent(name)}/100`
}