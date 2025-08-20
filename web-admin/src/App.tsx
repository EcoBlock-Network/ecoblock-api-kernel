import React from 'react'
import Blocks from './pages/Blocks'

export default function App() {
  return (
    <div className="app">
      <header className="header">
        <h1>EcoBlock — Backoffice</h1>
      </header>
      <main className="main">
        <Blocks />
      </main>
    </div>
  )
}
