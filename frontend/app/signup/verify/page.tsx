'use client'

import { Suspense, useEffect, useState } from 'react'
import Link from 'next/link'
import { useSearchParams } from 'next/navigation'

type ViewState = 'idle' | 'loading' | 'success' | 'error'

function SignupVerifyContent() {
  const searchParams = useSearchParams()
  const token = searchParams.get('token') || ''
  const [state, setState] = useState<ViewState>('idle')
  const [message, setMessage] = useState('Paste the verification link from your inbox.')

  useEffect(() => {
    if (!token) {
      setState('error')
      setMessage('Missing verification token.')
      return
    }

    const verify = async () => {
      setState('loading')
      try {
        const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
        const response = await fetch(`${apiUrl}/auth/signup/verify`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ token })
        })
        const data = await response.json().catch(() => ({ message: 'Verification failed' }))
        if (response.ok && data.status === 'verified') {
          setState('success')
          setMessage(data.message || 'Account verified.')
        } else {
          setState('error')
          setMessage(data.message || 'Verification failed.')
        }
      } catch (error) {
        console.error('Verification error:', error)
        setState('error')
        setMessage('Network error. Try again.')
      }
    }

    verify()
  }, [token])

  return (
    <section className="box">
      <h2 className="section-title">Status</h2>
      <p className={`status ${state === 'error' ? 'error' : state === 'success' ? 'success' : 'warning'}`}>{message}</p>
      <div className="actions">
        <Link className="button" href="/login">
          Go to login
        </Link>
        <Link className="button ghost" href="/signup">
          Start over
        </Link>
      </div>
    </section>
  )
}

export default function SignupVerifyPage() {
  return (
    <main className="app">
      <header className="header">
        <h1>W9 Mail / Verify signup</h1>
        <p>Finalize your registration and start sending.</p>
      </header>

      <nav className="nav">
        <Link className="nav-link" href="/">
          Composer
        </Link>
        <Link className="nav-link" href="/manage">
          Manage
        </Link>
        <Link className="nav-link" href="/docs">
          Docs
        </Link>
        <Link className="nav-link" href="/profile">
          Profile
        </Link>
        <Link className="nav-link" href="/login">
          Login
        </Link>
        <Link className="nav-link active" href="/signup/verify">
          Verify
        </Link>
      </nav>

      <Suspense fallback={<section className="box"><p>Loading…</p></section>}>
        <SignupVerifyContent />
      </Suspense>
    </main>
  )
}

