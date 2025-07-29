import React from "react"

import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@lana/web/ui/breadcrumb"

interface BaseBreadcrumbLink {
  title: string | React.ReactNode
}

interface ClickableBreadcrumbLink extends BaseBreadcrumbLink {
  isCurrentPage?: false
  href: string
}

interface CurrentPageBreadcrumbLink extends BaseBreadcrumbLink {
  isCurrentPage: true
  href?: never
}

export type BreadcrumbLink = ClickableBreadcrumbLink | CurrentPageBreadcrumbLink

interface FlexibleBreadcrumbProps {
  links: BreadcrumbLink[]
}

const BreadCrumbWrapper = ({ links }: FlexibleBreadcrumbProps) => {
  return (
    <Breadcrumb className="px-2 hidden md:block">
      <BreadcrumbList>
        {links.map((link, index) => (
          <React.Fragment key={index}>
            <BreadcrumbItem>
              {link.isCurrentPage ? (
                <BreadcrumbPage
                  className="flex items-center gap-3 align-middle"
                  tabIndex={-1}
                  aria-hidden="true"
                >
                  {link.title}
                </BreadcrumbPage>
              ) : (
                <BreadcrumbLink
                  href={link.href}
                  className="flex items-center gap-3 align-middle"
                  tabIndex={-1}
                  aria-hidden="true"
                >
                  {link.title}
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
            {index < links.length - 1 && <BreadcrumbSeparator />}
          </React.Fragment>
        ))}
      </BreadcrumbList>
    </Breadcrumb>
  )
}

export const ListPageBreadcrumb = ({ currentPage }: { currentPage: string }) => {
  const links: BreadcrumbLink[] = [
    { title: "Dashboard", href: "/dashboard" },
    { title: currentPage, isCurrentPage: true },
  ]

  return <BreadCrumbWrapper links={links} />
}

export { BreadCrumbWrapper }
