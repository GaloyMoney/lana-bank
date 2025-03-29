import * as React from "react"
import { NumericFormat, NumericFormatProps } from "react-number-format"

import { cn } from "@lana/web/utils"

const BaseInput = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, ...props }, ref) => (
  <input
    className={cn(
      "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
      className,
    )}
    ref={ref}
    {...props}
  />
))
BaseInput.displayName = "BaseInput"

interface CustomInputProps extends React.ComponentProps<"input"> {
  onValueChange?: (value: string) => void
}

const Input = React.forwardRef<HTMLInputElement, CustomInputProps>(
  ({ className, type, onValueChange, onChange, ...props }, ref) => {
    if (type === "number") {
      return (
        <NumericFormat
          customInput={BaseInput}
          className={className}
          thousandSeparator
          allowNegative={false}
          getInputRef={ref}
          displayType="input"
          onValueChange={(values) => {
            onValueChange?.(values.value)
            if (onChange) {
              const event = {
                target: {
                  value: values.value,
                  name: props.name,
                },
              } as React.ChangeEvent<HTMLInputElement>
              onChange(event)
            }
          }}
          {...(props as NumericFormatProps)}
        />
      )
    }

    return (
      <BaseInput
        type={type}
        className={className}
        ref={ref}
        onChange={onChange}
        {...props}
      />
    )
  },
)

Input.displayName = "Input"

export { Input }
