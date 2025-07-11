"use client"

import { toast } from "sonner"

export const PublicIdBadge = ({ publicId }: { publicId: string }) => {
  return (
    <span
      className="font-mono text-sm cursor-pointer"
      onClick={() => {
        navigator.clipboard.writeText(publicId)
        toast.success("Public ID copied to clipboard")
      }}
    >
      #{publicId}
    </span>
  )
}
