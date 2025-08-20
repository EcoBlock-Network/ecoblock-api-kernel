import React, { useEffect, useState } from 'react'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3000'

function getToken(): string | null {
  try { return localStorage.getItem('ecoblock_token') } catch (_) { return null }
}

type Blog = {
  id: string
  title: string
  slug: string
  body: string
  author: string
  created_at?: string
}

export default function Blogs() {
  const [items, setItems] = useState<Blog[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function fetchList() {
    setLoading(true); setError(null)
    try {
      const headers: Record<string,string> = { 'Accept': 'application/json' }
      const token = getToken(); if (token) headers['Authorization'] = `Bearer ${token}`
      const res = await fetch(`${API_BASE}/communication/blog`, { headers })
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`)
      const data = await res.json()
      setItems(data.items || [])
    } catch (e: any) { setError(e.message || String(e)) }
    finally { setLoading(false) }
  }

  useEffect(() => { fetchList() }, [])

  async function createDemo() {
    const payload = { title: 'Hello from CMS', slug: `hello-${Date.now()}`, body: 'body', author: 'admin' }
    const token = getToken()
    const headers: Record<string,string> = { 'Content-Type': 'application/json' }
    if (token) headers['Authorization'] = `Bearer ${token}`
    const res = await fetch(`${API_BASE}/communication/blog`, { method: 'POST', headers, body: JSON.stringify(payload) })
    if (!res.ok) { const txt = await res.text(); alert('create failed: '+txt); return }
    await fetchList()
  }

  async function del(id: string) {
    if (!confirm('Supprimer ce billet ?')) return
    const token = getToken(); const headers: Record<string,string> = {}
    if (token) headers['Authorization'] = `Bearer ${token}`
    const res = await fetch(`${API_BASE}/communication/blog/${id}`, { method: 'DELETE', headers })
    if (!res.ok) { alert('delete failed'); return }
    await fetchList()
  }

  return (
    <section>
      <div className="toolbar flex items-center space-x-2 mb-4">
        <button className="px-3 py-1 bg-gray-200 rounded" onClick={() => fetchList()}>Rafraîchir</button>
        <button className="px-3 py-1 bg-blue-600 text-white rounded" onClick={() => createDemo()}>Créer démo</button>
      </div>

      {loading && <p>Chargement...</p>}
      {error && <p className="text-red-600">Erreur: {error}</p>}

      <div className="card">
        <table className="min-w-full text-sm">
          <thead>
            <tr className="text-left text-gray-600 border-b">
              <th className="py-2">Title</th>
              <th className="py-2">Author</th>
              <th className="py-2">Slug</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody>
            {items.map(b => (
              <tr key={b.id} className="border-b hover:bg-gray-50">
                <td className="py-2">{b.title}</td>
                <td className="py-2 text-gray-600">{b.author}</td>
                <td className="py-2 text-sm text-gray-500">{b.slug}</td>
                <td className="py-2">
                  <button className="mr-2 px-2 py-1 bg-gray-200 rounded text-sm" onClick={() => navigator.clipboard.writeText(b.slug)}>Copier slug</button>
                  <button className="mr-2 px-2 py-1 bg-blue-600 text-white rounded text-sm" onClick={() => alert('Preview not implemented')}>Preview</button>
                  <button className="px-2 py-1 bg-red-600 text-white rounded text-sm" onClick={() => del(b.id)}>Supprimer</button>
                </td>
              </tr>
            ))}
            {items.length === 0 && (
              <tr>
                <td colSpan={4} className="py-6 text-center text-gray-500">Aucun billet trouvé</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  )
}
