'use client'

import { useEffect, useRef } from 'react'

declare global {
  interface Window {
    turnstile?: {
      render: (element: string | HTMLElement, options: {
        sitekey: string
        callback?: (token: string) => void
        'error-callback'?: () => void
        'expired-callback'?: () => void
      }) => string
      reset: (widgetId?: string) => void
      remove: (widgetId?: string) => void
    }
  }
}

interface TurnstileProps {
  onVerify: (token: string) => void
  onError?: () => void
}

export default function Turnstile({ onVerify, onError }: TurnstileProps) {
  const widgetRef = useRef<HTMLDivElement>(null)
  const widgetIdRef = useRef<string | null>(null)
  const siteKey = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || ''

  useEffect(() => {
    if (!siteKey || !widgetRef.current) return

    const checkTurnstile = () => {
      if (window.turnstile && widgetRef.current && !widgetIdRef.current) {
        widgetIdRef.current = window.turnstile.render(widgetRef.current, {
          sitekey: siteKey,
          callback: (token: string) => {
            onVerify(token)
          },
          'error-callback': () => {
            if (onError) onError()
          },
          'expired-callback': () => {
            if (widgetIdRef.current) {
              window.turnstile?.reset(widgetIdRef.current)
            }
          }
        })
      } else if (!window.turnstile) {
        setTimeout(checkTurnstile, 100)
      }
    }

    checkTurnstile()

    return () => {
      if (widgetIdRef.current && window.turnstile) {
        window.turnstile.remove(widgetIdRef.current)
        widgetIdRef.current = null
      }
    }
  }, [onVerify, onError, siteKey])

  if (!siteKey) {
    return null
  }

  return <div ref={widgetRef} style={{ marginTop: '1rem' }}></div>
}

