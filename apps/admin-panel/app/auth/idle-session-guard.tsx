"use client"

import { useCallback, useEffect, useRef, useState } from "react"
import { useIdleTimer, type EventsType } from "react-idle-timer"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"

import { keycloak as getKeycloak, logout as keycloakLogout } from "./keycloak"

const IDLE_MS = 5 * 60 * 1000
const PROMPT_MS = 30 * 1000
const ACTION_THROTTLE_MS = 10 * 1000

const IDLE_EVENTS: EventsType[] = [
  "pointerdown",
  "keydown",
  "wheel",
  "touchstart",
  "scroll",
  "mousemove",
  "visibilitychange",
]

export default function IdleSessionGuard() {
  const t = useTranslations("Auth.IdleSessionDialog")
  const [showPrompt, setShowPrompt] = useState(false)
  const [countdownSeconds, setCountdownSeconds] = useState(PROMPT_MS / 1000)
  const countdownIntervalRef = useRef<number | null>(null)
  const lastRefreshAtRef = useRef<number>(0)
  const mountedRef = useRef(true)

  const clearCountdown = useCallback(() => {
    if (countdownIntervalRef.current !== null) {
      window.clearInterval(countdownIntervalRef.current)
      countdownIntervalRef.current = null
    }
  }, [])

  const performLogout = useCallback(async () => {
    setShowPrompt(false)
    clearCountdown()
    try {
      await keycloakLogout()
    } catch (err) {
      console.error("Logout error:", err)
      window.location.reload()
    }
  }, [clearCountdown])

  const startCountdown = useCallback(() => {
    setShowPrompt(true)
    const deadline = Date.now() + PROMPT_MS
    setCountdownSeconds(Math.ceil(PROMPT_MS / 1000))
    clearCountdown()
    countdownIntervalRef.current = window.setInterval(() => {
      if (!mountedRef.current) return
      const remaining = Math.max(0, Math.ceil((deadline - Date.now()) / 1000))
      setCountdownSeconds(remaining)
      if (remaining <= 0) {
        clearCountdown()
        performLogout()
      }
    }, 1000)
  }, [clearCountdown, performLogout])

  const refreshToken = useCallback(
    async (opts: { force?: boolean } = {}) => {
      const now = Date.now()
      if (!opts.force && now - lastRefreshAtRef.current < ACTION_THROTTLE_MS) return
      if (document.visibilityState !== "visible") return

      const kc = await getKeycloak()
      if (!kc) return
      try {
        await kc.updateToken(-1)
        lastRefreshAtRef.current = now
      } catch (err) {
        console.error("Token refresh failed:", err)
        await performLogout()
      }
    },
    [performLogout],
  )

  const { reset } = useIdleTimer({
    timeout: IDLE_MS,
    promptBeforeIdle: PROMPT_MS,
    crossTab: true,
    events: showPrompt ? [] : IDLE_EVENTS,
    eventsThrottle: ACTION_THROTTLE_MS,
    onPrompt: () => {
      startCountdown()
    },
    onIdle: () => performLogout(),
    onAction: (event) => {
      if (showPrompt) return
      if (event && "isTrusted" in event && !event.isTrusted) return
      refreshToken()
    },
  })

  const handleStaySignedIn = useCallback(async () => {
    setShowPrompt(false)
    clearCountdown()
    await refreshToken({ force: true })
    reset()
  }, [clearCountdown, refreshToken, reset])

  useEffect(() => {
    mountedRef.current = true
    return () => {
      mountedRef.current = false
      clearCountdown()
    }
  }, [clearCountdown])

  return (
    <Dialog
      open={showPrompt}
      onOpenChange={(open) => {
        if (!open) {
          handleStaySignedIn()
        }
      }}
    >
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>
            {t("description", { seconds: countdownSeconds })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="flex-col gap-2 sm:flex-col sm:space-y-2 sm:space-x-0">
          <Button variant="outline" onClick={performLogout} className="w-full">
            {t("buttons.logout")}
          </Button>
          <Button onClick={handleStaySignedIn} className="w-full">
            {t("buttons.staySignedIn")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
