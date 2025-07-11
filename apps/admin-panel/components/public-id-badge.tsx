"use client"

import { toast } from "sonner"
import { Badge } from "@lana/web/ui/badge"
import { cn } from "@lana/web/utils"

import type { PublicIdTarget } from "@/lib/graphql/generated"

type EntityType = NonNullable<PublicIdTarget["__typename"]>

const entityStyles: Record<
  EntityType,
  { bg: string; text: string; border: string; hover: string }
> = {
  Customer: {
    bg: "bg-green-100",
    text: "text-green-800",
    border: "border-green-300",
    hover: "hover:bg-green-200",
  },
}

export interface PublicIdBadgeProps {
  publicId: string
  entityType: EntityType
  className?: string
}

async function copyToClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
    toast.success("Public ID copied to clipboard")
  } catch (err) {
    toast.error("Failed to copy to clipboard")
  }
}

export const PublicIdBadge: React.FC<PublicIdBadgeProps> = ({
  publicId,
  entityType,
  className,
}) => {
  const styles = entityStyles[entityType]

  return (
    <Badge
      variant="secondary"
      onClick={() => copyToClipboard(publicId)}
      className={cn(
        "font-mono cursor-pointer transition-colors",
        styles.bg,
        styles.text,
        styles.border,
        styles.hover,
        className,
      )}
    >
      <span className="font-mono">{publicId}</span>
    </Badge>
  )
}
