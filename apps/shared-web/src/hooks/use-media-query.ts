"use client"
import { useEffect, useState } from "react"

const BREAKPOINTS = {
  "sm": "640px",
  "md": "768px",
  "lg": "1024px",
  "xl": "1280px",
  "2xl": "1536px",
} as const

type BreakpointKey = keyof typeof BREAKPOINTS

export function useMediaQuery(breakpoint: BreakpointKey | string): boolean {
  const [matches, setMatches] = useState(false)

  useEffect(() => {
    const query = BREAKPOINTS[breakpoint as BreakpointKey]
      ? `(min-width: ${BREAKPOINTS[breakpoint as BreakpointKey]})`
      : breakpoint

    const media = window.matchMedia(query)
    setMatches(media.matches)

    const listener = (event: MediaQueryListEvent) => {
      setMatches(event.matches)
    }

    media.addEventListener("change", listener)

    return () => {
      media.removeEventListener("change", listener)
    }
  }, [breakpoint])

  return matches
}

export function useBreakpointUp(breakpoint: BreakpointKey): boolean {
  return useMediaQuery(breakpoint)
}

export function useBreakpointDown(breakpoint: BreakpointKey): boolean {
  const nextBreakpointValue = BREAKPOINTS[breakpoint]
  return useMediaQuery(`(max-width: ${nextBreakpointValue})`)
}
