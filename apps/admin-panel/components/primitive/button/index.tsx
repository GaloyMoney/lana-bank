import React from "react"
import { cva, VariantProps } from "class-variance-authority"
import { Slot, Slottable } from "@radix-ui/react-slot"

import { LoadingSpinner } from "../loading-spinner"

import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap p-2 px-4 rounded-xl text-sm font-medium transition focus:outline-none disabled:cursor-not-allowed",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-button-text hover:bg-primary-disabled disabled:bg-primary-disabled",
        secondary:
          "bg-secondary hover:bg-secondary-disabled disabled:bg-secondary-disabled",
        ghost: "bg-transparent hover:bg-secondary",
        transparent: "bg-transparent",
        link: "bg-transparent hover:underline",
        outline: "bg-transparent border-2 border-primary",
      },
      enabled: {
        true: "opacity-100 cursor-pointer",
        false: "opacity-50 cursor-not-allowed",
      },
    },
    defaultVariants: {
      variant: "primary",
      enabled: true,
    },
  },
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
  loading?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ loading = false, children, className, variant, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"
    return (
      <Comp
        ref={ref}
        disabled={loading}
        className={cn(
          buttonVariants({ variant, className }),
          loading && "relative text-transparent",
        )}
        {...props}
      >
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center">
            <LoadingSpinner />
          </div>
        )}
        <Slottable>{children}</Slottable>
      </Comp>
    )
  },
)
Button.displayName = "Button"

export { Button }
