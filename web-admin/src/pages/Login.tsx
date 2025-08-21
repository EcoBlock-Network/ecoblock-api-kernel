import React, { useState } from 'react'
import { useToast } from '../lib/ToastProvider'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3000'

function getRedirectTarget() {
  try {
    const params = new URLSearchParams(window.location.search)
    return params.get('redirect')
  } catch { return null }
}

export default function Login({ onLogin }: { onLogin: (token: string) => void }) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const toast = useToast()

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      const res = await fetch(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      })
      const text = await res.text()
      if (!res.ok) {
        const data = JSON.parse(text || '{}').catch?.(() => null)
        toast.showApiError(data || text)
        setLoading(false)
        return
      }
      const data = JSON.parse(text)
      const token = data.token || data.access_token || data.accessToken
      if (!token) { toast.showApiError({ error: 'no token returned' }); setLoading(false); return }
      try { localStorage.setItem('ecoblock_token', token) } catch (_) {}
      onLogin(token)
      // If redirect is provided, navigate there (simple client-side)
      const redirect = getRedirectTarget()
      if (redirect) {
        window.history.replaceState({}, '', redirect)
        window.location.reload()
      }
    } catch (err: any) {
      setError(err.message || String(err))
      setLoading(false)
    }
  }

  return (
    <section>
      <div className="card max-w-md mx-auto p-6">
        <div className="mb-4 text-center">
          <div className="mb-2 text-2xl font-semibold">EcoBlock</div>
          <div className="text-sm text-gray-500">Backoffice</div>
        </div>

        <form onSubmit={submit} className="space-y-4">
          <div>
            <label className="block text-sm mb-1">Username</label>
            <input className="w-full p-2 border rounded" value={username} onChange={e => setUsername(e.target.value)} />
          </div>
          <div>
            <label className="block text-sm mb-1">Password</label>
            <input className="w-full p-2 border rounded" type="password" value={password} onChange={e => setPassword(e.target.value)} />
          </div>
          <div className="flex items-center justify-between">
            <button type="submit" className="btn" disabled={loading}>{loading ? 'Connexion...' : 'Se connecter'}</button>
            <a href="#" className="text-sm text-gray-500">Mot de passe oublié?</a>
          </div>
          {error && <p className="text-red-600 text-sm">{error}</p>}
        </form>
      </div>
    </section>
  )
}
