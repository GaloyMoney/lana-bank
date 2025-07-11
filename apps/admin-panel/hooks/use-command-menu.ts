import { useState } from "react"

export function useCommandMenu() {
  const [open, setOpen] = useState(false)

  const openCommandMenu = () => setOpen(true)
  const closeCommandMenu = () => setOpen(false)
  const toggleCommandMenu = () => setOpen((prev) => !prev)

  return {
    open,
    setOpen,
    openCommandMenu,
    closeCommandMenu,
    toggleCommandMenu,
  }
}
