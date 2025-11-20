'use client'

import { useState } from 'react'
import Link from 'next/link'
import { useSession } from '../../lib/session'
import Nav from '../components/Nav'

export default function ProfilePage() {
  const { session, logout } = useSession()
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [sendingReset, setSendingReset] = useState(false)

  const requestReset = async () => {
    if (!session?.email) return
    setSendingReset(true)
    setMessage(null)
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/auth/password-reset`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: session.email })
      })
      const data = await response.json().catch(() => ({ message: 'Request sent' }))
      if (response.ok) {
        setMessage({ type: 'success', text: data.message || 'Reset email dispatched.' })
      } else {
        setMessage({ type: 'error', text: data.message || 'Unable to send reset email.' })
      }
    } catch (error) {
      console.error('Failed to request reset:', error)
      setMessage({ type: 'error', text: 'Network error. Please try again.' })
    } finally {
      setSendingReset(false)
    }
  }

  return (
    <main className="app">
      <header className="header">
        <h1>W9 Mail / Profile</h1>
        <p>Review your access and trigger password resets.</p>
      </header>

      <Nav active="profile" />

      {message && <div className={`status ${message.type}`}>{message.text}</div>}

      {!session ? (
        <section className="box">
          <h2 className="section-title">Authentication required</h2>
          <p>Sign in to see your profile.</p>
          <Link className="button" href="/login">
            Go to login
          </Link>
        </section>
      ) : (
        <>
          <section className="box">
            <h2 className="section-title">Account details</h2>
            <ul className="list">
              <li>Email: {session.email}</li>
              <li>Role: {session.role}</li>
              <li>Must change password: {session.mustChangePassword ? 'Yes' : 'No'}</li>
            </ul>
            <div className="actions">
              <button className="button subtle" onClick={logout}>
                Sign out
              </button>
            </div>
          </section>

          <section className="box">
            <h2 className="section-title">Password reset</h2>
            <p>We email a secure link from the default sender. Follow it within 30 minutes to set a new password.</p>
            <button className="button" onClick={requestReset} disabled={sendingReset}>
              {sendingReset ? 'Sending…' : 'Send reset email'}
            </button>
            <p className="hint">
              Already have a token? Go to the <Link href="/reset-password">reset portal</Link>.
            </p>
          </section>
        </>
      )}
    </main>
  )
}

