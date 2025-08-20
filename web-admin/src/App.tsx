import React, { useEffect, useState } from 'react'
import Blocks from './pages/Blocks'
import Blogs from './pages/Blogs'
import Login from './pages/Login'

export default function App() {
  const [token, setToken] = useState<string | null>(null)
  const [page, setPage] = useState<'blocks'|'blogs'>('blocks')

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
    // If not authenticated show a focused login screen without header/sidebar
    token ? (
      <div className="min-h-screen flex">
        <aside className="sidebar">
          <div className="mb-6">
            <h2 className="text-lg font-semibold">EcoBlock</h2>
            <p className="text-sm text-gray-500">Backoffice</p>
          </div>
          <nav className="space-y-2">
            <button className="w-full text-left" onClick={() => setPage('blocks')}>Blocks</button>
            <button className="w-full text-left" onClick={() => setPage('blogs')}>Blogs</button>
          </nav>
        </aside>
        <div className="flex-1">
          <header className="header">
            <h1 className="text-base font-medium">EcoBlock — {page === 'blocks' ? 'Blocks' : 'Blogs'}</h1>
            {token ? <div><button className="btn" onClick={logout}>Logout</button></div> : null}
          </header>
          <main className="content">
            {page === 'blocks' ? <Blocks /> : <Blogs />}
          </main>
        </div>
      </div>
    ) : (
      <div className="min-h-screen flex items-center justify-center">
        <div className="max-w-md w-full px-4">
          <Login onLogin={onLogin} />
        </div>
      </div>
    )
  )
}
