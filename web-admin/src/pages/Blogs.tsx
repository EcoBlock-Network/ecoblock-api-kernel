import React, { useEffect, useState } from 'react'
import ReactQuill from 'react-quill'
import 'react-quill/dist/quill.snow.css'
import { useToast } from '../lib/ToastProvider'

const API_BASE = import.meta.env.VITE_API_BASE ?? '/api'

function getToken(): string | null {
  try { return sessionStorage.getItem('ecoblock_token') } catch (_) { return null }
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
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(10)
  const [totalPages, setTotalPages] = useState(1)
  const [totalCount, setTotalCount] = useState<number | null>(null)
  const [maxFileSizeMB] = useState(10)
  const [uploadProgress, setUploadProgress] = useState(0)
  const toast = useToast()
  const [query, setQuery] = useState('')

  async function fetchList(p: number = page, q: string = query) {
  toast.setLoading(true); setError(null)
    try {
      const headers: Record<string,string> = { 'Accept': 'application/json' }
      const token = getToken(); if (token) headers['Authorization'] = `Bearer ${token}`
      const params = new URLSearchParams()
      params.set('page', String(p))
      params.set('per_page', String(perPage))
      if (q) params.set('q', q)
    const res = await fetch(`${API_BASE}/communication/blog?${params.toString()}`, { headers })
    if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
  const data = await res.json()
  setItems(data.items || [])
  setTotalCount(data.total ?? null)
  setPage(data.page || p)
  setTotalPages(data.total_pages || 1)
    } catch (e: any) { setError(e.message || String(e)) }
  finally { toast.setLoading(false) }
  }

  useEffect(() => { fetchList(1, query) }, [])

  async function createDemo() {
    const payload = { title: 'Hello from CMS', slug: `hello-${Date.now()}`, body: 'body', author: 'admin' }
    const token = getToken()
    const headers: Record<string,string> = { 'Content-Type': 'application/json' }
    if (token) headers['Authorization'] = `Bearer ${token}`
    const res = await fetch(`${API_BASE}/communication/blog`, { method: 'POST', headers, body: JSON.stringify(payload) })
  if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
    await fetchList()
  }

  // open editor for new or existing
  function openEditor(b?: Blog) {
    if (b) setEditing(b)
    else setEditing({ id: '', title: '', slug: '', body: '', author: 'admin' })
  }

  function closeEditor() { setEditing(null) }

  async function saveBlog(b: Blog) {
    // simple client-side validation
    const title = (b.title || '').trim()
    const slug = (b.slug || '').trim()
    const body = (b.body || '').trim()
    if (title.length < 3) { toast.showToast('title must be at least 3 characters', 'error'); return }
    if (!slug) { toast.showToast('slug is required', 'error'); return }
    if (body.length < 10) { toast.showToast('body must be at least 10 characters', 'error'); return }

    const token = getToken()
    const headers: Record<string,string> = { 'Content-Type': 'application/json' }
    if (token) headers['Authorization'] = `Bearer ${token}`
    if (b.id) {
    const res = await fetch(`${API_BASE}/communication/blog/${b.id}`, { method: 'PUT', headers, body: JSON.stringify({ title, slug, body }) })
  if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
    } else {
    const res = await fetch(`${API_BASE}/communication/blog`, { method: 'POST', headers, body: JSON.stringify({ title, slug, body, author: b.author }) })
  if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
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
  if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
    await fetchList()
  }

  function onSearch(e?: React.FormEvent) { e?.preventDefault(); fetchList(1, query) }

  return (
    <section>
      <div className="toolbar flex items-center space-x-2 mb-4">
        <form onSubmit={onSearch} className="flex items-center space-x-2">
          <input value={query} onChange={e => setQuery(e.target.value)} placeholder="Search title..." className="p-2 border rounded" />
          <button className="px-3 py-1 bg-gray-200 rounded" type="submit">Search</button>
        </form>
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

      <div className="flex items-center justify-between mt-4">
        <div>Page {page} / {totalPages} {totalCount !== null ? `— ${totalCount} billets` : ''}</div>
        <div className="space-x-2 flex items-center">
          <label className="text-sm mr-2">Par page</label>
          <select className="px-2 py-1 border rounded mr-4" value={perPage} onChange={e => { const v = Number(e.target.value); setPerPage(v); fetchList(1, query) }}>
            <option value={5}>5</option>
            <option value={10}>10</option>
            <option value={25}>25</option>
          </select>
          <button className="px-2 py-1 border rounded" disabled={page<=1} onClick={() => fetchList(page-1, query)}>Prev</button>
          <button className="px-2 py-1 border rounded" disabled={page>=totalPages} onClick={() => fetchList(page+1, query)}>Next</button>
        </div>
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
              <label className="block text-sm mb-1">Body</label>
              <div className="mb-2">
                <ReactQuill
                  theme="snow"
                  value={editing.body}
                  onChange={(v: string) => setEditing({ ...editing, body: v })}
                  modules={{
                    toolbar: {
                      container: [['bold','italic'], ['link','image']],
                      handlers: {
                        image: async function() {
                          const input = document.createElement('input')
                          input.setAttribute('type', 'file')
                          input.setAttribute('accept', 'image/*')
                          input.onchange = async () => {
                            const file = input.files && input.files[0]
                            if (!file) return
                            if (file.size > maxFileSizeMB * 1024 * 1024) { toast.showToast(`Fichier trop gros (max ${maxFileSizeMB}MB)`, 'error'); return }
                            const token = getToken()
                            const fd = new FormData(); fd.append('file', file, file.name)
                            try {
                              // use XHR to get progress if desired
                              const xhr = new XMLHttpRequest()
                              const url = `${API_BASE}/communication/upload`
                              xhr.open('POST', url)
                              if (token) xhr.setRequestHeader('Authorization', `Bearer ${token}`)
                              const data = await new Promise<any>((resolve, reject) => {
                                xhr.onload = () => {
                                  if (xhr.status >= 200 && xhr.status < 300) resolve(JSON.parse(xhr.responseText))
                                  else reject(new Error(`${xhr.status} ${xhr.statusText}`))
                                }
                                xhr.onerror = () => reject(new Error('network error'))
                                xhr.send(fd)
                              })
                              const urlRes = data.uploaded && data.uploaded[0]
                              if (urlRes) {
                                const range = (this as any).quill.getSelection(true)
                                ;(this as any).quill.insertEmbed(range.index, 'image', urlRes)
                                toast.showToast('Upload réussi', 'success')
                              }
                            } catch (err: any) { toast.showToast('upload failed: '+(err.message||String(err)), 'error') }
                          }
                          input.click()
                        }
                      }
                    }
                  }}
                />
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <input type="file" onChange={async e => {
                const f = e.target.files && e.target.files[0]; if (!f) return;
                if (f.size > maxFileSizeMB * 1024 * 1024) { toast.showToast(`Fichier trop gros (max ${maxFileSizeMB}MB)`, 'error'); return }
                const token = getToken();
                const fd = new FormData(); fd.append('file', f, f.name);
                setUploadProgress(0)
                setUploading(true)
                try {
                  const url = `${API_BASE}/communication/upload`
                  const xhr = new XMLHttpRequest()
                  xhr.open('POST', url)
                  if (token) xhr.setRequestHeader('Authorization', `Bearer ${token}`)
                  xhr.upload.onprogress = (ev) => { if (ev.lengthComputable) setUploadProgress(Math.round((ev.loaded/ev.total)*100)) }
                  const data = await new Promise<any>((resolve, reject) => {
                    xhr.onload = () => {
                      if (xhr.status >= 200 && xhr.status < 300) {
                        resolve(JSON.parse(xhr.responseText))
                      } else {
                        reject(new Error(`${xhr.status} ${xhr.statusText}`))
                      }
                    }
                    xhr.onerror = () => reject(new Error('network error'))
                    xhr.send(fd)
                  })
                  const urlRes = data.uploaded && data.uploaded[0]
                  if (urlRes && editing) {
                    setEditing({ ...editing, body: editing.body + `<img src="${urlRes}" alt="upload" />` })
                    toast.showToast('Upload réussi', 'success')
                  }
                } catch (err: any) { toast.showToast('upload failed: '+(err.message||String(err)), 'error') }
                finally { setUploading(false); setUploadProgress(0) }
              }} />
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
