export type ToastType = 'info' | 'success' | 'error'

function ensureContainer() {
  let c = document.getElementById('ecoblock-toasts')
  if (!c) {
    c = document.createElement('div')
    c.id = 'ecoblock-toasts'
    c.style.position = 'fixed'
    c.style.right = '16px'
    c.style.bottom = '16px'
    c.style.zIndex = '9999'
    c.style.display = 'flex'
    c.style.flexDirection = 'column'
    c.style.gap = '8px'
    document.body.appendChild(c)
  }
  return c
}

export function showToast(message: string, type: ToastType = 'info', timeout = 3500) {
  const c = ensureContainer()
  const el = document.createElement('div')
  el.textContent = message
  el.style.minWidth = '200px'
  el.style.padding = '10px 12px'
  el.style.borderRadius = '8px'
  el.style.color = '#fff'
  el.style.boxShadow = '0 4px 12px rgba(0,0,0,0.12)'
  el.style.fontSize = '13px'
  if (type === 'success') el.style.background = '#16A34A'
  else if (type === 'error') el.style.background = '#DC2626'
  else el.style.background = '#374151'

  c.appendChild(el)
  const t = setTimeout(() => {
    el.style.opacity = '0'
    setTimeout(() => el.remove(), 200)
  }, timeout)

  el.addEventListener('click', () => { clearTimeout(t); el.remove() })
  return () => { clearTimeout(t); el.remove() }
}
