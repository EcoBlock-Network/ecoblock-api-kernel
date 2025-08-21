import React, { useEffect, useState } from 'react'
import { useToast } from '../lib/ToastProvider'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3000'

function getToken(): string | null {
  try { return localStorage.getItem('ecoblock_token') } catch (_) { return null }
}

type User = {
  id: string
  username: string
  email: string
  is_admin: boolean
  created_at?: string
}

export default function Users() {
  const [items, setItems] = useState<User[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [creating, setCreating] = useState(false)
  const [creatingAdmin, setCreatingAdmin] = useState(false)
  const toast = useToast()

  async function fetchList() {
    setLoading(true); setError(null)
    try {
      const headers: Record<string,string> = { 'Accept': 'application/json' }
      const token = getToken(); if (token) headers['Authorization'] = `Bearer ${token}`
      const res = await fetch(`${API_BASE}/users`, { headers })
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`)
      const data = await res.json()
      setItems(data.items || data || [])
    } catch (e: any) { setError(e.message || String(e)) }
    finally { setLoading(false) }
  }

  useEffect(() => { fetchList() }, [])

  async function createUser() {
    setCreating(true)
    toast.setLoading(true)
    try {
      const token = getToken()
      const headers: Record<string,string> = { 'Content-Type': 'application/json' }
      if (token) headers['Authorization'] = `Bearer ${token}`
      const payload = { username, email, password }
      const res = await fetch(`${API_BASE}/users`, { method: 'POST', headers, body: JSON.stringify(payload) })
    if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
      setUsername(''); setEmail(''); setPassword('')
      toast.showToast('Utilisateur créé', 'success')
      await fetchList()
    } catch (e: any) { toast.showToast('create failed: '+(e.message||String(e)), 'error') }
    finally { setCreating(false); toast.setLoading(false) }
  }

  async function createAdmin() {
    setCreatingAdmin(true); toast.setLoading(true)
    try {
      const token = getToken()
      const headers: Record<string,string> = { 'Content-Type': 'application/json' }
      if (token) headers['Authorization'] = `Bearer ${token}`
      const payload = { username, email, password }
      const res = await fetch(`${API_BASE}/users/admin`, { method: 'POST', headers, body: JSON.stringify(payload) })
    if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
      setUsername(''); setEmail(''); setPassword('')
      toast.showToast('Admin créé', 'success')
      await fetchList()
    } catch (e: any) { toast.showToast('create admin failed: '+(e.message||String(e)), 'error') }
    finally { setCreatingAdmin(false); toast.setLoading(false) }
  }

  async function makeAdmin(id: string) {
    if (!confirm('Grant admin to this user?')) return
    toast.setLoading(true)
    try {
      const token = getToken(); const headers: Record<string,string> = {}
      if (token) headers['Authorization'] = `Bearer ${token}`
      const res = await fetch(`${API_BASE}/users/${id}/grant_admin`, { method: 'POST', headers })
    if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
      toast.showToast('Utilisateur promu admin', 'success')
      await fetchList()
    } catch (e: any) { toast.showToast('grant admin failed: '+(e.message||String(e)), 'error') }
    finally { toast.setLoading(false) }
  }

  return (
    <section>
      <div className="toolbar flex items-center space-x-2 mb-4">
        <button className="px-3 py-1 bg-gray-200 rounded" onClick={() => fetchList()}>Rafraîchir</button>
      </div>

      {loading && <p>Chargement...</p>}
      {error && <p className="text-red-600">Erreur: {error}</p>}

      <div className="card mb-4 p-4">
        <h3 className="font-semibold mb-2">Créer utilisateur</h3>
        <div className="grid grid-cols-3 gap-2">
          <input className="p-2 border rounded" placeholder="username" value={username} onChange={e => setUsername(e.target.value)} />
          <input className="p-2 border rounded" placeholder="email" value={email} onChange={e => setEmail(e.target.value)} />
          <input type="password" className="p-2 border rounded" placeholder="password" value={password} onChange={e => setPassword(e.target.value)} />
        </div>
        <div className="mt-2 space-x-2">
          <button className="px-3 py-1 bg-blue-600 text-white rounded" onClick={() => createUser()} disabled={creating}>Créer</button>
          <button className="px-3 py-1 bg-green-600 text-white rounded" onClick={() => createAdmin()} disabled={creatingAdmin}>Créer admin</button>
        </div>
      </div>

      <div className="card">
        <table className="min-w-full text-sm">
          <thead>
            <tr className="text-left text-gray-600 border-b">
              <th className="py-2">Username</th>
              <th className="py-2">Email</th>
              <th className="py-2">Admin</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody>
            {items.map(u => (
              <tr key={u.id} className="border-b hover:bg-gray-50">
                <td className="py-2">{u.username}</td>
                <td className="py-2 text-gray-600">{u.email}</td>
                <td className="py-2 text-sm text-gray-500">{u.is_admin ? 'Yes' : 'No'}</td>
                <td className="py-2">
                  {!u.is_admin && (
                    <button className="px-2 py-1 bg-yellow-500 text-white rounded text-sm" onClick={() => makeAdmin(u.id)}>Make admin</button>
                  )}
                </td>
              </tr>
            ))}
            {items.length === 0 && (
              <tr>
                <td colSpan={4} className="py-6 text-center text-gray-500">Aucun utilisateur trouvé</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  )
}
