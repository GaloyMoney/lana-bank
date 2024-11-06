import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center justify-center rounded-xl border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2",
  {
    variants: {
      variant: {
        success: "border-[#49dc35] text-[#b8f8ab] bg-[#35dc433c]",
        default: "border-[#007bff] text-[#7abaff] bg-[#007bff0d]",
        secondary: "border-[#6c757d] text-[#939596] bg-[#6c757d0d]",
        destructive: "border-[#dc3545] text-[#e9a1a8] bg-[#dc35463d]",
        outline: "border-[#343a40] text-[#343a40] bg-[#343a400d]",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {
  blink?: boolean
}

function Badge({ className, variant, blink, ...props }: BadgeProps) {
  return (
    <div
      className={cn(
        "relative flex items-center justify-center text-center",
        badgeVariants({ variant }),
        className,
      )}
      {...props}
    >
      {blink && (
        <span
          className={cn("mr-1 h-2 w-2 rounded-full animate-pulse", {
            "bg-[#25ff08]": variant === "success",
            "bg-[#15c4ff]": variant === "default",
            "bg-[#6c757d]": variant === "secondary",
            "bg-[#ff182f]": variant === "destructive",
            "bg-[#343a40]": variant === "outline",
          })}
        />
      )}
      {props.children}
    </div>
  )
}

Badge.displayName = "Badge"
export { Badge, badgeVariants }
