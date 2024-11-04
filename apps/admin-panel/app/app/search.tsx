"use client"

import { useEffect, useRef, useState } from "react"
import { HiSearch } from "react-icons/hi"

import { Input } from "@/components"

const SearchInput: React.FC = () => {
  const inputRef = useRef<HTMLInputElement>(null)
  const [isMacOS, setIsMacOS] = useState(false)

  useEffect(() => {
    const userAgent = navigator.userAgent || navigator.vendor || (window as any).opera
    const isMac = /Macintosh|Mac OS X/i.test(userAgent)
    setIsMacOS(isMac)

    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.key === "k" && (event.ctrlKey || event.metaKey)) || event.key === "/") {
        event.preventDefault()
        inputRef.current?.focus()
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

  return (
    <Input
      ref={inputRef}
      type="text"
      placeholder="Search for Customer, Credit Facility, or Menu Items"
      leftNode={<HiSearch className="text-placeholder" />}
      rightNode={
        <div className="flex items-center text-placeholder text-body-sm">
          <span className="mr-1">{isMacOS ? "⌘" : "Ctrl"}</span> + K or /
        </div>
      }
    />
  )
}

export { SearchInput }
