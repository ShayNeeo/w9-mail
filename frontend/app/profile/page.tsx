'use client'

import { useState } from 'react'
import Link from 'next/link'
import { useSession } from '../../lib/session'
import Nav from '../components/Nav'

export default function ProfilePage() {
  const { session, logout } = useSession()
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [changingPassword, setChangingPassword] = useState(false)
  const [passwordForm, setPasswordForm] = useState({ old: '', new: '', confirm: '' })
  const [changing, setChanging] = useState(false)

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!session?.token) return
    
    if (passwordForm.new !== passwordForm.confirm) {
      setMessage({ type: 'error', text: 'New passwords do not match' })
      return
    }
    if (passwordForm.new.length < 8) {
      setMessage({ type: 'error', text: 'Password must be at least 8 characters' })
      return
    }

    setChanging(true)
    setMessage(null)
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/auth/change-password`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.token}`
        },
        body: JSON.stringify({
          current_password: passwordForm.old,
          new_password: passwordForm.new
        })
      })
      const data = await response.json().catch(() => ({ message: 'Failed to change password' }))
      if (response.ok) {
        setMessage({ type: 'success', text: data.message || 'Password updated successfully' })
        setPasswordForm({ old: '', new: '', confirm: '' })
        setChangingPassword(false)
      } else {
        setMessage({ type: 'error', text: data.message || data.error || 'Failed to change password' })
      }
    } catch (error) {
      console.error('Failed to change password:', error)
      setMessage({ type: 'error', text: 'Network error. Please try again.' })
    } finally {
      setChanging(false)
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
            <h2 className="section-title">Change Password</h2>
            {!changingPassword ? (
              <>
                <p>Update your password. You'll need to provide your current password.</p>
                <button className="button" onClick={() => setChangingPassword(true)}>
                  Change Password
                </button>
              </>
            ) : (
              <form className="form" onSubmit={handleChangePassword}>
                <div className="row">
                  <label>Old Password</label>
                  <input
                    type="password"
                    value={passwordForm.old}
                    onChange={(e) => setPasswordForm({ ...passwordForm, old: e.target.value })}
                    required
                  />
                </div>
                <div className="row">
                  <label>New Password</label>
                  <input
                    type="password"
                    value={passwordForm.new}
                    onChange={(e) => setPasswordForm({ ...passwordForm, new: e.target.value })}
                    required
                    minLength={8}
                  />
                </div>
                <div className="row">
                  <label>Confirm New Password</label>
                  <input
                    type="password"
                    value={passwordForm.confirm}
                    onChange={(e) => setPasswordForm({ ...passwordForm, confirm: e.target.value })}
                    required
                    minLength={8}
                  />
                </div>
                <button className="button" type="submit" disabled={changing}>
                  {changing ? 'Changing…' : 'Change Password'}
                </button>
                <button
                  className="button subtle"
                  type="button"
                  onClick={() => {
                    setChangingPassword(false)
                    setPasswordForm({ old: '', new: '', confirm: '' })
                    setMessage(null)
                  }}
                >
                  Cancel
                </button>
              </form>
            )}
            <p className="hint">
              Forgot your password? Go to the <Link href="/login">login page</Link> to request a reset.
            </p>
          </section>
        </>
      )}
    </main>
  )
}

