'use client'

import { useState } from 'react'
import Link from 'next/link'
import Nav from '../components/Nav'
import Turnstile from '../components/Turnstile'

export default function SignupPage() {
  const [form, setForm] = useState({ email: '', password: '' })
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [loading, setLoading] = useState(false)
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const siteKey = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY
    if (siteKey && !turnstileToken) {
      setMessage({ type: 'error', text: 'Please complete the security check' })
      return
    }
    setLoading(true)
    setMessage(null)
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/auth/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...form, turnstile_token: turnstileToken })
      })
      const data = await response.json().catch(() => ({ message: 'Failed to register' }))
      if (response.ok && data.status === 'pending') {
        setMessage({ type: 'success', text: data.message || 'Verification email sent.' })
        setForm({ email: '', password: '' })
      } else {
        setMessage({ type: 'error', text: data.message || 'Signup failed' })
      }
    } catch (error) {
      console.error('Failed to sign up:', error)
      setMessage({ type: 'error', text: 'Network error. Please try again.' })
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="app">
      <header className="header">
        <h1>W9 Mail / Signup</h1>
        <p>Register a normal user account and confirm ownership via email.</p>
      </header>

      <Nav active="signup" />

      {message && <div className={`status ${message.type}`}>{message.text}</div>}

      <section className="box">
        <h2 className="section-title">Create account</h2>
        <form className="form" onSubmit={handleSubmit}>
          <div className="row">
            <label>Email</label>
            <input
              type="email"
              value={form.email}
              onChange={(e) => setForm({ ...form, email: e.target.value })}
              required
            />
          </div>
          <div className="row">
            <label>Password</label>
            <input
              type="password"
              value={form.password}
              onChange={(e) => setForm({ ...form, password: e.target.value })}
              required
              minLength={8}
            />
            <small>Minimum 8 characters. You can rotate it later from Profile.</small>
          </div>
          <Turnstile 
            onVerify={(token) => setTurnstileToken(token)}
            onError={() => setTurnstileToken(null)}
          />
          <button className="button" type="submit" disabled={loading}>
            {loading ? 'Submitting…' : 'Create account'}
          </button>
        </form>
        <p className="hint">
          Already registered? <Link href="/login">Return to login</Link>.
        </p>
      </section>
    </main>
  )
}

