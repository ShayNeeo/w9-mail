import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'W9 Mail - Email Service API',
  description: 'Email service API provider using Microsoft SMTP IMAP POP3',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}

