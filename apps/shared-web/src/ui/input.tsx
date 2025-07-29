import * as React from "react";
import { NumericFormat, NumericFormatProps } from "react-number-format";

import { cn } from "@lana/web/utils";

const BaseInput = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, ...props }, ref) => (
  <input
    className={cn(
      "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
      className
    )}
    ref={ref}
    {...props}
  />
));
BaseInput.displayName = "BaseInput";

interface CustomInputProps extends React.ComponentProps<"input"> {
  onValueChange?: (value: string) => void;
  startAdornment?: React.ReactNode;
  endAdornment?: React.ReactNode;
}

const Input = React.forwardRef<HTMLInputElement, CustomInputProps>(
  (
    { className, type, onValueChange, onChange, startAdornment, endAdornment, ...props },
    ref
  ) => {
    const renderInput = (inputProps: any) => {
      if (startAdornment || endAdornment) {
        return (
          <div className="flex h-9 w-full rounded-md border border-input bg-transparent shadow-sm transition-colors focus-within:ring-1 focus-within:ring-ring">
            {startAdornment && (
              <div className="flex items-center rounded-l-md bg-muted px-3 text-sm text-muted-foreground font-mono border-r border-input">
                {startAdornment}
              </div>
            )}

            {React.cloneElement(inputProps.element, {
              ...inputProps.element.props,
              className: cn(
                "flex-1 bg-transparent px-3 py-1 text-base placeholder:text-muted-foreground focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground transition-colors",
                "!border-0 !shadow-none !ring-0 focus-visible:!ring-0",
                !startAdornment && !endAdornment && "rounded-md",
                startAdornment && !endAdornment && "rounded-r-md",
                !startAdornment && endAdornment && "rounded-l-md",
                startAdornment && endAdornment && "rounded-none"
              ),
              style: {
                ...inputProps.element.props.style,
                paddingLeft: undefined,
                paddingRight: undefined,
                boxShadow: "none",
              },
            })}

            {endAdornment && (
              <div className="flex items-center rounded-r-md bg-muted px-3 text-sm text-muted-foreground font-mono border-l border-input">
                {endAdornment}
              </div>
            )}
          </div>
        );
      }
      return inputProps.element;
    };
    if (type === "number") {
      const numericInput = (
        <NumericFormat
          customInput={BaseInput}
          className={className}
          thousandSeparator
          allowNegative={false}
          getInputRef={ref}
          displayType="input"
          onValueChange={(values) => {
            onValueChange?.(values.value);
            if (onChange) {
              const event = {
                target: {
                  value: values.value,
                  name: props.name,
                },
              } as React.ChangeEvent<HTMLInputElement>;
              onChange(event);
            }
          }}
          {...(props as NumericFormatProps)}
        />
      );
      return renderInput({ element: numericInput });
    }

    const baseInput = (
      <BaseInput
        type={type}
        className={className}
        ref={ref}
        onChange={onChange}
        {...props}
      />
    );
    return renderInput({ element: baseInput });
  }
);

Input.displayName = "Input";

export { Input };
