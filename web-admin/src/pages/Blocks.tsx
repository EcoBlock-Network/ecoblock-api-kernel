import React, { useEffect, useState } from 'react'
import { useToast } from '../lib/ToastProvider'


type Block = {
  id: string
  parents: string[]
  data: any
  public_key?: string
  created_at?: string
}

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3000'

function getToken(): string | null {
  try {
    return localStorage.getItem('ecoblock_token');
  } catch (_) {
    return null;
  }
}

export default function Blocks() {
  const [blocks, setBlocks] = useState<Block[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const toast = useToast()

  async function fetchBlocks() {
    setLoading(true)
    setError(null)
    try {
      const headers: Record<string,string> = { 'Accept': 'application/json' }
      const token = getToken()
      if (token) headers['Authorization'] = `Bearer ${token}`
  const res = await fetch(`${API_BASE}/tangle/blocks`, { headers })
  if (!res.ok) { const data = await res.json().catch(() => null); toast.showApiError(data || await res.text()); return }
  const data = await res.json()
      const items = Array.isArray(data) ? data : data.items ?? data.blocks ?? []
      setBlocks(items)
    } catch (err: any) {
      setError(err.message || String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchBlocks()
  }, [])

  return (
    <section>
      <div className="toolbar flex items-center space-x-2 mb-4">
        <button className="px-3 py-1 bg-gray-200 rounded" onClick={() => fetchBlocks()}>Rafraîchir</button>
      </div>

      {loading && <p>Chargement...</p>}
      {error && <p className="error">Erreur: {error}</p>}

      {!loading && !error && (
        <div className="blocks">
          {blocks && blocks.length > 0 ? (
            blocks.map(b => (
              <article key={b.id} className="block">
                <div className="meta">
                  <strong>{b.id}</strong>
                  <time>{b.created_at}</time>
                </div>
                <div className="parents">Parents: {b.parents?.join(', ')}</div>
                <pre className="data">{JSON.stringify(b.data, null, 2)}</pre>
                <div className="mt-2">
                  <button className="px-2 py-1 bg-blue-600 text-white rounded text-sm mr-2" onClick={() => navigator.clipboard.writeText(b.id)}>Copier ID</button>
                </div>
              </article>
            ))
          ) : (
            <p>Aucun block trouvé.</p>
          )}
        </div>
      )}
    </section>
  )
}
