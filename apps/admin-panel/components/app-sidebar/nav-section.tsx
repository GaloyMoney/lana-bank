"use client"

import { LucideIcon } from "lucide-react"
import Link from "next/link"
import { usePathname } from "next/navigation"

import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  useSidebar,
} from "@lana/web/ui/sidebar"

export type NavItem = {
  title: string
  url: string
  icon: LucideIcon
}

export type NavSectionProps = {
  items: NavItem[]
  label?: string
}

export function NavSection({ items, label }: NavSectionProps) {
  const pathname = usePathname()
  const { setOpenMobile, isMobile } = useSidebar()

  if (items.length === 0) return null

  const handleClick = () => {
    if (isMobile) {
      setOpenMobile(false)
    }
  }

  return (
    <SidebarGroup>
      {label && <SidebarGroupLabel>{label}</SidebarGroupLabel>}
      <SidebarMenu>
        {items.map((item) => {
          const Icon = item.icon
          const isActive = pathname?.startsWith(item.url)

          return (
            <SidebarMenuItem key={item.url}>
              <SidebarMenuButton asChild tooltip={item.title} isActive={isActive}>
                <Link href={item.url} prefetch={true} onClick={handleClick} tabIndex={-1}>
                  <Icon className="h-4 w-4" />
                  <span className="text-base md:text-sm">{item.title}</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )
        })}
      </SidebarMenu>
    </SidebarGroup>
  )
}
