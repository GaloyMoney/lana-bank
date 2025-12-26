"use client"

import { useEffect, useState } from "react"
import { Search } from "lucide-react"
import { useTranslations } from "next-intl"

import { Input } from "@lana/web/ui/input"

interface SearchAndCommandProps {
  onOpenCommandPalette: () => void
}

export function SearchAndCommand({ onOpenCommandPalette }: SearchAndCommandProps) {
  const t = useTranslations("CommandMenu")
  const [isMac, setIsMac] = useState(false)

  useEffect(() => {
    const macPlatforms = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"]
    const userAgent = window.navigator.userAgent
    const platform = window.navigator.platform

    setIsMac(
      macPlatforms.includes(platform) ||
        userAgent.includes("Mac") ||
        /Mac/.test(navigator.platform),
    )
  }, [])

  return (
    <div className="relative flex-1 max-w-xs">
      <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        type="text"
        placeholder={t("placeholders.whatDoYouNeed")}
        readOnly
        onClick={onOpenCommandPalette}
        className="pl-10 pr-16 w-full cursor-pointer bg-muted"
      />
      <kbd className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
        <span className="text-xs">{isMac ? "⌘" : "Ctrl"}</span>
        <span className="text-xs">K</span>
      </kbd>
    </div>
  )
}
