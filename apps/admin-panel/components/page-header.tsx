"use client"

import { DynamicBreadcrumb } from "@/app/dynamic-breadcrumb"

interface PageHeaderProps {
  title: string
  description?: string
  actions?: React.ReactNode
  showBreadcrumb?: boolean
}

export function PageHeader({
  title,
  description,
  actions,
  showBreadcrumb = true,
}: PageHeaderProps) {
  return (
    <div className="p-4 border-b space-y-1">
      {showBreadcrumb && <DynamicBreadcrumb />}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">{title}</h1>
          {description && (
            <p className="text-sm text-muted-foreground">{description}</p>
          )}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </div>
  )
}
