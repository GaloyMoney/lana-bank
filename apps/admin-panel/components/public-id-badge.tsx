"use client"

import { Badge } from "@lana/web/ui/badge"

export const PublicIdBadge = ({ publicId }: { publicId: string }) => {
  return (
    <Badge
      variant="secondary"
      className="font-mono text-[0.7rem] text-muted-foreground py-0 px-[10px] "
    >
      {publicId}
    </Badge>
  )
}
