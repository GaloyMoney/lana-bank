import * as React from "react"
import { IoChevronDown } from "react-icons/io5"

import { cn } from "@/lib/utils"

export interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {}

const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, ...props }, ref) => {
    return (
      <div className="relative w-full">
        <select
          className={cn(
            "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors",
            "text-foreground placeholder:text-muted-foreground",
            "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
            "disabled:cursor-not-allowed disabled:opacity-50",
            "appearance-none pr-8",
            className,
          )}
          ref={ref}
          {...props}
        >
          {props.children}
        </select>
        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
          <IoChevronDown className="h-4 w-4 text-muted-foreground" />
        </div>
      </div>
    )
  },
)
Select.displayName = "Select"

export { Select }
