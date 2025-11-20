'use client'

import Link from 'next/link'
import { useSession } from '../../lib/session'

interface NavProps {
  active?: string
}

export default function Nav({ active }: NavProps) {
  const { session, logout } = useSession()

  const links = [
    { href: '/', id: 'home', label: 'Composer' },
    { href: '/manage', id: 'manage', label: 'Manage' },
    { href: '/docs', id: 'docs', label: 'Docs' },
    { href: '/profile', id: 'profile', label: 'Profile' },
    { href: '/reset-password', id: 'reset-password', label: 'Reset' },
  ]

  return (
    <nav className="nav">
      {links.map((l) => (
        <Link key={l.id} className={`nav-link ${active === l.id ? 'active' : ''}`} href={l.href}>
          {l.label}
        </Link>
      ))}
      {!session && (
        <>
          <Link className={`nav-link ${active === 'login' ? 'active' : ''}`} href="/login">
            Login
          </Link>
          <Link className={`nav-link ${active === 'signup' ? 'active' : ''}`} href="/signup">
            Signup
          </Link>
        </>
      )}
      {session && (
        <button type="button" className="nav-link" onClick={logout}>
          Sign out
        </button>
      )}
    </nav>
  )
}
