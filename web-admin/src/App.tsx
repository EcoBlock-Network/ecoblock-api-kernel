import React, { useEffect, useState } from 'react'
import Blocks from './pages/Blocks'
import Login from './pages/Login'

export default function App() {
  const [token, setToken] = useState<string | null>(null)

  useEffect(() => {
    try {
      const t = localStorage.getItem('ecoblock_token')
      if (t) setToken(t)
      // Support auto-login in dev via VITE_DEV_TOKEN
      const dev = (import.meta.env as any).VITE_DEV_TOKEN
      if (!t && dev) {
        localStorage.setItem('ecoblock_token', dev)
        setToken(dev)
      }
    } catch (_) {}
  }, [])

  function onLogin(t: string) {
    setToken(t)
  }

  function logout() {
    try { localStorage.removeItem('ecoblock_token') } catch (_) {}
    setToken(null)
  }

  return (
    <div className="app">
      <header className="header">
        <h1>EcoBlock — Backoffice</h1>
        {token && <button onClick={logout}>Logout</button>}
      </header>
      <main className="main">
        {token ? <Blocks /> : <Login onLogin={onLogin} />}
      </main>
    </div>
  )
}
