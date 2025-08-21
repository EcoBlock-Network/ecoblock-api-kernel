import React from 'react'
import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { ToastProvider, useToast } from '../ToastProvider'

function Demo() {
  const t = useToast()
  React.useEffect(() => { t.showToast('hello', 'success') }, [])
  return <div>demo</div>
}

describe('ToastProvider', () => {
  it('renders children and shows toasts', async () => {
    render(
      <ToastProvider>
        <Demo />
      </ToastProvider>
    )
    expect(screen.getByText('demo')).toBeTruthy()
    // toast appears with text
    const toast = await screen.findByText('hello')
    expect(toast).toBeTruthy()
  })
})
