import * as React from "react";
import { NumericFormat, NumericFormatProps } from "react-number-format";
import { cn } from "@lana/web/utils";
import { InputGroup, InputGroupAddon } from "@lana/web/ui/input-group";

interface InputProps extends React.ComponentProps<"input"> {
  startAdornment?: React.ReactNode;
  endAdornment?: React.ReactNode;
  containerClassName?: string;
}

function Input({
  className,
  type,
  startAdornment,
  endAdornment,
  containerClassName,
  ...props
}: InputProps) {
  const hasAdornments = startAdornment || endAdornment;

  const renderInput = (useGroupControl: boolean = false) => {
    const dataSlot = useGroupControl ? "input-group-control" : "input";
    const baseClasses = useGroupControl
      ? "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 flex h-9 w-full min-w-0 rounded-md bg-transparent px-3 py-1 text-base transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
      : "file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive";

    if (type === "number") {
      const { onChange, name, ...rest } =
        props as React.InputHTMLAttributes<HTMLInputElement>;
      return (
        <NumericFormat
          type="text"
          data-slot={dataSlot}
          className={cn(baseClasses, className)}
          thousandSeparator
          allowNegative={false}
          onValueChange={(v) => {
            if (onChange) {
              onChange({
                target: { value: v.value, name },
              } as unknown as React.ChangeEvent<HTMLInputElement>);
            }
          }}
          {...(rest as unknown as NumericFormatProps)}
        />
      );
    }

    return (
      <input
        type={type}
        data-slot={dataSlot}
        className={cn(baseClasses, className)}
        {...props}
      />
    );
  };

  if (!hasAdornments) {
    return renderInput(false);
  }

  return (
    <InputGroup className={containerClassName}>
      {startAdornment && (
        <InputGroupAddon align="inline-start">{startAdornment}</InputGroupAddon>
      )}
      {renderInput(true)}
      {endAdornment && (
        <InputGroupAddon align="inline-end">{endAdornment}</InputGroupAddon>
      )}
    </InputGroup>
  );
}

export { Input };
