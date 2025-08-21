import React, { createContext, useCallback, useContext, useMemo, useState } from 'react'

export type ToastType = 'info' | 'success' | 'error'
type Toast = { id: string; message: string; type: ToastType }

type ToastContext = {
  showToast: (message: string, type?: ToastType, timeout?: number) => void
  setLoading: (v: boolean) => void
}

const ctx = createContext<ToastContext | null>(null)

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])
  const [loading, setLoading] = useState(false)

  const showToast = useCallback((message: string, type: ToastType = 'info', timeout = 3500) => {
    const id = String(Date.now()) + Math.random().toString(36).slice(2, 8)
    const t: Toast = { id, message, type }
    setToasts(s => [t, ...s])
    setTimeout(() => setToasts(s => s.filter(x => x.id !== id)), timeout)
  }, [])

  const value = useMemo(() => ({ showToast, setLoading }), [showToast])

  return (
    <ctx.Provider value={value}>
      {children}
      {/* toasts container */}
      <div style={{ position: 'fixed', right: 16, bottom: 16, zIndex: 9999, display: 'flex', flexDirection: 'column', gap: 8 }}>
        {toasts.map(t => (
          <div key={t.id} style={{ minWidth: 200, padding: '10px 12px', borderRadius: 8, color: '#fff', boxShadow: '0 4px 12px rgba(0,0,0,0.12)', fontSize: 13, background: t.type === 'success' ? '#16A34A' : t.type === 'error' ? '#DC2626' : '#374151' }}>
            {t.message}
          </div>
        ))}
      </div>

      {/* global spinner overlay */}
      {loading && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.25)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 9998 }}>
          <div style={{ padding: 16, background: '#fff', borderRadius: 8, boxShadow: '0 6px 20px rgba(0,0,0,0.12)' }}>
            <div className="loader">Chargement...</div>
          </div>
        </div>
      )}
    </ctx.Provider>
  )
}

export function useToast() {
  const c = useContext(ctx)
  if (!c) throw new Error('useToast must be used within ToastProvider')
  return c
}
