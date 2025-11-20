'use client'

import { useState, useEffect } from 'react'

interface EmailAccount {
  id: string
  email: string
  displayName: string
  isActive: boolean
}

export default function ManagePage() {
  const [accounts, setAccounts] = useState<EmailAccount[]>([])
  const [loading, setLoading] = useState(true)
  const [formData, setFormData] = useState({
    email: '',
    displayName: '',
    password: '',
    isActive: true
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
        setAccounts(data)
      }
    } catch (error) {
      console.error('Failed to fetch accounts:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/accounts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData)
      })
      if (response.ok) {
        fetchAccounts()
        setFormData({ email: '', displayName: '', password: '', isActive: true })
      }
    } catch (error) {
      console.error('Failed to create account:', error)
    }
  }

  const toggleActive = async (id: string, isActive: boolean) => {
    try {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || '/api'
      const response = await fetch(`${apiUrl}/accounts/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ isActive: !isActive })
      })
      if (response.ok) {
        fetchAccounts()
      }
    } catch (error) {
      console.error('Failed to update account:', error)
    }
  }

  if (loading) return <div>Loading...</div>

  return (
    <main style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
      <h1>Manage Email Services</h1>
      
      <section style={{ marginTop: '2rem' }}>
        <h2>Add New Email Account</h2>
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxWidth: '500px' }}>
          <input
            type="email"
            placeholder="Email address"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            required
          />
          <input
            type="text"
            placeholder="Display name"
            value={formData.displayName}
            onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
            required
          />
          <input
            type="password"
            placeholder="Password"
            value={formData.password}
            onChange={(e) => setFormData({ ...formData, password: e.target.value })}
            required
          />
          <label>
            <input
              type="checkbox"
              checked={formData.isActive}
              onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
            />
            Active
          </label>
          <button type="submit">Add Account</button>
        </form>
      </section>

      <section style={{ marginTop: '3rem' }}>
        <h2>Email Accounts</h2>
        <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: '1rem' }}>
          <thead>
            <tr>
              <th style={{ textAlign: 'left', padding: '0.5rem', borderBottom: '1px solid #ddd' }}>Email</th>
              <th style={{ textAlign: 'left', padding: '0.5rem', borderBottom: '1px solid #ddd' }}>Display Name</th>
              <th style={{ textAlign: 'left', padding: '0.5rem', borderBottom: '1px solid #ddd' }}>Status</th>
              <th style={{ textAlign: 'left', padding: '0.5rem', borderBottom: '1px solid #ddd' }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {accounts.map((account) => (
              <tr key={account.id}>
                <td style={{ padding: '0.5rem', borderBottom: '1px solid #ddd' }}>{account.email}</td>
                <td style={{ padding: '0.5rem', borderBottom: '1px solid #ddd' }}>{account.displayName}</td>
                <td style={{ padding: '0.5rem', borderBottom: '1px solid #ddd' }}>
                  {account.isActive ? 'Active' : 'Inactive'}
                </td>
                <td style={{ padding: '0.5rem', borderBottom: '1px solid #ddd' }}>
                  <button onClick={() => toggleActive(account.id, account.isActive)}>
                    {account.isActive ? 'Deactivate' : 'Activate'}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </main>
  )
}

