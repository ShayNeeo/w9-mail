'use client'

import { useState, useEffect } from 'react'
import Link from 'next/link'

interface EmailAccount {
  id: string
  email: string
  displayName: string
  isActive: boolean
}

export default function Home() {
  const [accounts, setAccounts] = useState<EmailAccount[]>([])
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState(false)
  const [message, setMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null)
  const [formData, setFormData] = useState({
    from: '',
    to: '',
    cc: '',
    bcc: '',
    subject: '',
    body: ''
  })

  useEffect(() => {
    fetchAccounts()
  }, [])

  const fetchAccounts = async () => {
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/accounts`)
      if (response.ok) {
        const data = await response.json()
        setAccounts(data.filter((acc: EmailAccount) => acc.isActive))
      }
    } catch (error) {
      console.error('Failed to fetch accounts:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSending(true)
    setMessage(null)

    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/send`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          from: formData.from,
          to: formData.to,
          subject: formData.subject,
          body: formData.body,
          cc: formData.cc || undefined,
          bcc: formData.bcc || undefined
        })
      })

      if (response.ok) {
        setMessage({ type: 'success', text: 'Email sent successfully!' })
        setFormData({
          from: formData.from,
          to: '',
          cc: '',
          bcc: '',
          subject: '',
          body: ''
        })
      } else {
        const error = await response.json().catch(() => ({ message: 'Failed to send email' }))
        setMessage({ type: 'error', text: error.message || 'Failed to send email' })
      }
    } catch (error) {
      setMessage({ type: 'error', text: 'Network error. Please try again.' })
    } finally {
      setSending(false)
    }
  }

  if (loading) {
    return (
      <main style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
        <div>Loading...</div>
      </main>
    )
  }

  return (
    <main style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2rem' }}>
        <h1>W9 Mail - Send Email</h1>
        <nav style={{ display: 'flex', gap: '1rem' }}>
          <Link href="/manage">Manage Accounts</Link>
          <Link href="/docs">API Docs</Link>
        </nav>
      </div>

      {message && (
        <div style={{
          padding: '1rem',
          marginBottom: '1rem',
          borderRadius: '4px',
          backgroundColor: message.type === 'success' ? '#d4edda' : '#f8d7da',
          color: message.type === 'success' ? '#155724' : '#721c24',
          border: `1px solid ${message.type === 'success' ? '#c3e6cb' : '#f5c6cb'}`
        }}>
          {message.text}
        </div>
      )}

      <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        <div>
          <label htmlFor="from" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            From (Sender) *
          </label>
          <select
            id="from"
            value={formData.from}
            onChange={(e) => setFormData({ ...formData, from: e.target.value })}
            required
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px' }}
          >
            <option value="">Select sender account</option>
            {accounts.map((account) => (
              <option key={account.id} value={account.email}>
                {account.displayName} ({account.email})
              </option>
            ))}
          </select>
        </div>

        <div>
          <label htmlFor="to" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            To (Receiver) *
          </label>
          <input
            type="email"
            id="to"
            value={formData.to}
            onChange={(e) => setFormData({ ...formData, to: e.target.value })}
            placeholder="recipient@example.com"
            required
            multiple
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px' }}
          />
          <small style={{ color: '#666' }}>Separate multiple emails with commas</small>
        </div>

        <div>
          <label htmlFor="cc" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            CC (Carbon Copy)
          </label>
          <input
            type="email"
            id="cc"
            value={formData.cc}
            onChange={(e) => setFormData({ ...formData, cc: e.target.value })}
            placeholder="cc@example.com"
            multiple
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px' }}
          />
          <small style={{ color: '#666' }}>Separate multiple emails with commas</small>
        </div>

        <div>
          <label htmlFor="bcc" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            BCC (Blind Carbon Copy)
          </label>
          <input
            type="email"
            id="bcc"
            value={formData.bcc}
            onChange={(e) => setFormData({ ...formData, bcc: e.target.value })}
            placeholder="bcc@example.com"
            multiple
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px' }}
          />
          <small style={{ color: '#666' }}>Separate multiple emails with commas</small>
        </div>

        <div>
          <label htmlFor="subject" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            Title (Subject) *
          </label>
          <input
            type="text"
            id="subject"
            value={formData.subject}
            onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
            placeholder="Email subject"
            required
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px' }}
          />
        </div>

        <div>
          <label htmlFor="body" style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>
            Contents (Body) *
          </label>
          <textarea
            id="body"
            value={formData.body}
            onChange={(e) => setFormData({ ...formData, body: e.target.value })}
            placeholder="Email content"
            required
            rows={10}
            style={{ width: '100%', padding: '0.5rem', fontSize: '1rem', border: '1px solid #ddd', borderRadius: '4px', fontFamily: 'inherit' }}
          />
        </div>

        <button
          type="submit"
          disabled={sending || !formData.from || !formData.to || !formData.subject || !formData.body}
          style={{
            padding: '0.75rem 2rem',
            fontSize: '1rem',
            fontWeight: 'bold',
            color: 'white',
            backgroundColor: sending ? '#ccc' : '#007bff',
            border: 'none',
            borderRadius: '4px',
            cursor: sending ? 'not-allowed' : 'pointer',
            alignSelf: 'flex-start'
          }}
        >
          {sending ? 'Sending...' : 'Send Email'}
        </button>
      </form>
    </main>
  )
}
