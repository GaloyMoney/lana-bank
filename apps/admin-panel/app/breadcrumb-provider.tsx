"use client"

import { createContext, useContext, useState } from "react"

import { usePathname } from "next/navigation"

import { validate } from "uuid"

import { useNavItems } from "@/components/app-sidebar"
import type { BreadcrumbLink } from "@/components/breadcrumb-wrapper"

interface BreadcrumbContextType {
  links: BreadcrumbLink[]
  setCustomLinks: (links: BreadcrumbLink[]) => void
  resetToDefault: () => void
}

const BreadcrumbContext = createContext<BreadcrumbContextType | undefined>(undefined)

function generateDefaultLinks(
  pathname: string,
  useNavItemsResult: ReturnType<typeof useNavItems>,
): BreadcrumbLink[] {
  const { findNavItemByUrl } = useNavItemsResult
  const segments = pathname.split("/").filter(Boolean)

  const dashboardItem = findNavItemByUrl("/dashboard")
  const links: BreadcrumbLink[] = []

  let currentPath = ""
  segments.forEach((segment, index) => {
    if (segment === "dashboard") return
    currentPath += "/" + segment
    const isLastSegment = index === segments.length - 1
    const navItem = findNavItemByUrl(currentPath)
    let title: string | React.ReactNode

    if (navItem) {
      title = navItem.title
    } else if (validate(segment)) {
      title = segment
    } else {
      title = segment
        .split("-")
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(" ")
    }

    links.push({
      title,
      ...(isLastSegment ? { isCurrentPage: true as const } : { href: currentPath }),
    })
  })

  if (pathname === "/" || pathname === "/dashboard") {
    links[0] = {
      title: dashboardItem?.title || "Dashboard",
      isCurrentPage: true as const,
    }
  }

  return links
}

export function BreadcrumbProvider({ children }: { children: React.ReactNode }) {
  const pathname = usePathname()
  const [customLinks, setCustomLinks] = useState<BreadcrumbLink[] | null>(null)

  const navItemsResult = useNavItems()
  const links = customLinks ?? generateDefaultLinks(pathname, navItemsResult)

  return (
    <BreadcrumbContext.Provider
      value={{
        links,
        setCustomLinks: (newLinks) => setCustomLinks(newLinks),
        resetToDefault: () => setCustomLinks(null),
      }}
    >
      {children}
    </BreadcrumbContext.Provider>
  )
}

export const useBreadcrumb = () => {
  const context = useContext(BreadcrumbContext)
  if (!context) {
    throw new Error("useBreadcrumb must be used within a BreadcrumbProvider")
  }
  return context
}
