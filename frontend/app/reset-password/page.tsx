'use client'

import { Suspense, useEffect, useState } from 'react'
import Link from 'next/link'
import { useSearchParams } from 'next/navigation'
import Turnstile from '../components/Turnstile'

function ResetPasswordContent() {
  const searchParams = useSearchParams()
  const initialToken = searchParams.get('token') || ''
  const [token, setToken] = useState(initialToken)
  const [password, setPassword] = useState('')
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [loading, setLoading] = useState(false)
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null)

  useEffect(() => {
    if (initialToken) {
      setToken(initialToken)
    }
  }, [initialToken])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!token.trim()) {
      setMessage({ type: 'error', text: 'Token required' })
      return
    }
    if (!password.trim()) {
      setMessage({ type: 'error', text: 'New password required' })
      return
    }
    const siteKey = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY
    if (siteKey && !turnstileToken) {
      setMessage({ type: 'error', text: 'Please complete the security check' })
      return
    }
    setLoading(true)
    setMessage(null)
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/auth/password-reset/confirm`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token, newPassword: password, turnstile_token: turnstileToken })
      })
      const data = await response.json().catch(() => ({ message: 'Reset failed' }))
      if (response.ok && data.status === 'success') {
        setMessage({ type: 'success', text: data.message || 'Password updated.' })
        setPassword('')
      } else {
        setMessage({ type: 'error', text: data.message || 'Reset failed' })
      }
    } catch (error) {
      console.error('Failed to reset password:', error)
      setMessage({ type: 'error', text: 'Network error. Please try again.' })
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      {message && <div className={`status ${message.type}`}>{message.text}</div>}
      <section className="box">
        <h2 className="section-title">Enter token + password</h2>
        <form className="form" onSubmit={handleSubmit}>
          <div className="row">
            <label>Token</label>
            <input type="text" value={token} onChange={(e) => setToken(e.target.value)} required />
            <small>The token lives in the reset link (…token=VALUE).</small>
          </div>
          <div className="row">
            <label>New password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={8}
            />
          </div>
          <Turnstile 
            onVerify={(token) => setTurnstileToken(token)}
            onError={() => setTurnstileToken(null)}
          />
          <button className="button" type="submit" disabled={loading}>
            {loading ? 'Resetting…' : 'Reset password'}
          </button>
        </form>
        <p className="hint">
          Didn&apos;t request this? Ignore the email or <Link href="/profile">trigger a new one</Link>.
        </p>
      </section>
    </>
  )
}

export default function ResetPasswordPage() {
  return (
    <main className="app">
      <header className="header">
        <h1>W9 Mail / Reset password</h1>
        <p>Use the token sent to your inbox to set a new password.</p>
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
        <Link className="nav-link" href="/signup">
          Signup
        </Link>
        <Link className="nav-link active" href="/reset-password">
          Reset
        </Link>
      </nav>

      <Suspense fallback={<section className="box"><p>Loading…</p></section>}>
        <ResetPasswordContent />
      </Suspense>
    </main>
  )
}

