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
  const [editing, setEditing] = useState<Blog | null>(null)
  const [preview, setPreview] = useState<Blog | null>(null)
  const [uploading, setUploading] = useState(false)

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

  // open editor for new or existing
  function openEditor(b?: Blog) {
    if (b) setEditing(b)
    else setEditing({ id: '', title: '', slug: '', body: '', author: 'admin' })
  }

  function closeEditor() { setEditing(null) }

  async function saveBlog(b: Blog) {
    const token = getToken()
    const headers: Record<string,string> = { 'Content-Type': 'application/json' }
    if (token) headers['Authorization'] = `Bearer ${token}`
    if (b.id) {
      const res = await fetch(`${API_BASE}/communication/blog/${b.id}`, { method: 'PUT', headers, body: JSON.stringify({ title: b.title }) })
      if (!res.ok) { alert('update failed'); return }
    } else {
      const res = await fetch(`${API_BASE}/communication/blog`, { method: 'POST', headers, body: JSON.stringify(b) })
      if (!res.ok) { const txt = await res.text(); alert('create failed: '+txt); return }
    }
    closeEditor()
    await fetchList()
  }

  // very small media helper: file -> base64 and insert as <img> into body
  async function uploadAndInsert(file: File) {
    setUploading(true)
    try {
      const reader = new FileReader()
      const p = new Promise<string>((res, rej) => {
        reader.onload = () => res(String(reader.result))
        reader.onerror = () => rej(reader.error)
      })
      reader.readAsDataURL(file)
      const dataUrl = await p
      if (!editing) return
      editing.body = editing.body + `\n<img src="${dataUrl}" alt="upload" />`
      setEditing({ ...editing })
    } finally { setUploading(false) }
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
  <button className="px-3 py-1 bg-green-600 text-white rounded" onClick={() => openEditor()}>Nouveau</button>
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
                  <button className="mr-2 px-2 py-1 bg-blue-600 text-white rounded text-sm" onClick={() => setPreview(b)}>Preview</button>
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

      {editing && (
        <div className="card mt-4 p-4">
          <h3 className="font-semibold mb-2">{editing.id ? 'Editer' : 'Créer'} billet</h3>
          <div className="space-y-2">
            <div>
              <label className="block text-sm mb-1">Title</label>
              <input className="w-full p-2 border rounded" value={editing.title} onChange={e => setEditing({ ...editing, title: e.target.value })} />
            </div>
            <div>
              <label className="block text-sm mb-1">Slug</label>
              <input className="w-full p-2 border rounded" value={editing.slug} onChange={e => setEditing({ ...editing, slug: e.target.value })} />
            </div>
            <div>
              <label className="block text-sm mb-1">Body (HTML allowed)</label>
              <textarea className="w-full p-2 border rounded" rows={8} value={editing.body} onChange={e => setEditing({ ...editing, body: e.target.value })} />
            </div>
            <div className="flex items-center space-x-2">
              <input type="file" onChange={e => e.target.files && uploadAndInsert(e.target.files[0])} />
              <button className="px-3 py-1 bg-blue-600 text-white rounded" onClick={() => saveBlog(editing)}>Enregistrer</button>
              <button className="px-3 py-1 bg-gray-200 rounded" onClick={() => closeEditor()}>Annuler</button>
            </div>
            {uploading && <div className="text-sm text-gray-500">Upload en cours...</div>}
          </div>
        </div>
      )}

      {preview && (
        <div className="card mt-4 p-4">
          <h3 className="font-semibold mb-2">Aperçu: {preview.title}</h3>
          <div className="prose max-w-none" dangerouslySetInnerHTML={{ __html: preview.body }} />
          <div className="mt-2"><button className="px-3 py-1 bg-gray-200 rounded" onClick={() => setPreview(null)}>Fermer</button></div>
        </div>
      )}
    </section>
  )
}
