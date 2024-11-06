import React, { ReactNode } from "react"

import { cn } from "@/lib/utils"

interface PageHeadingProps {
  children: ReactNode
  className?: string
}

const PageHeading = ({ children, className }: PageHeadingProps) => {
  return (
    <>
      <title>{children}</title>
      <h1
        className={cn(
          "scroll-m-20 text-3xl font-semibold tracking-tight first:mt-0 mb-8",
          className,
        )}
      >
        {children}
      </h1>
    </>
  )
}

export { PageHeading }
