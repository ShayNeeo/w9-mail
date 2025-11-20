import Link from 'next/link'

export default function Home() {
  return (
    <main style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
      <h1>W9 Mail - Email Service API</h1>
      <nav style={{ marginTop: '2rem', display: 'flex', gap: '1rem' }}>
        <Link href="/manage">Manage Email Services</Link>
        <Link href="/docs">API Documentation</Link>
      </nav>
    </main>
  )
}

