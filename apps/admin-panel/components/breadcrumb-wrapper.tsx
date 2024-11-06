import React from "react"
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/primitive/breadcrumb"

interface BaseBreadcrumbLink {
  title: string
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
    <Breadcrumb className="py-4 px-2">
      <BreadcrumbList>
        {links.map((link, index) => (
          <React.Fragment key={index}>
            <BreadcrumbItem>
              {link.isCurrentPage ? (
                <BreadcrumbPage>{link.title}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink href={link.href}>{link.title}</BreadcrumbLink>
              )}
            </BreadcrumbItem>
            {index < links.length - 1 && <BreadcrumbSeparator />}
          </React.Fragment>
        ))}
      </BreadcrumbList>
    </Breadcrumb>
  )
}

export { BreadCrumbWrapper }
