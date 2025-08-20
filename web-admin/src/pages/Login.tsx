import React, { useState } from 'react'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3000'

export default function Login({ onLogin }: { onLogin: (token: string) => void }) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    try {
      const res = await fetch(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      })
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`)
      const data = await res.json()
      const token = data.token || data.access_token || data.accessToken
      if (!token) throw new Error('no token returned')
      try { localStorage.setItem('ecoblock_token', token) } catch (_) {}
      onLogin(token)
    } catch (err: any) {
      setError(err.message || String(err))
    }
  }

  return (
    <section>
      <h2>Connexion</h2>
      <form onSubmit={submit} className="space-y-4 max-w-md">
        <div>
          <label className="block text-sm mb-1">Username</label>
          <input className="w-full p-2 border rounded" value={username} onChange={e => setUsername(e.target.value)} />
        </div>
        <div>
          <label className="block text-sm mb-1">Password</label>
          <input className="w-full p-2 border rounded" type="password" value={password} onChange={e => setPassword(e.target.value)} />
        </div>
        <button type="submit" className="btn">Se connecter</button>
        {error && <p className="error">{error}</p>}
      </form>
    </section>
  )
}
