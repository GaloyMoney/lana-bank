import * as React from "react";
import Link from "next/link";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { Spinner } from "@lana/web/ui/spinner";
import { cn } from "@lana/web/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all cursor-pointer disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive:
          "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60",
        outline:
          "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost:
          "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2 has-[>svg]:px-3",
        sm: "h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5",
        lg: "h-10 rounded-md px-6 has-[>svg]:px-4",
        icon: "size-9",
        "icon-sm": "size-8",
        "icon-lg": "size-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

type ButtonVariantProps = VariantProps<typeof buttonVariants>;

type CommonProps = {
  loading?: boolean;
  disabled?: boolean;
  className?: string;
  children?: React.ReactNode;
} & ButtonVariantProps;

type ButtonAsButton = CommonProps &
  Omit<React.ComponentProps<"button">, keyof CommonProps> & {
    href?: undefined;
    external?: undefined;
    asChild?: false;
  };

type ButtonAsLink = CommonProps &
  Omit<React.ComponentProps<"a">, keyof CommonProps | "href"> & {
    href: string;
    external?: boolean;
    asChild?: undefined;
  };

type ButtonAsChild = CommonProps &
  Omit<React.ComponentProps<"button">, keyof CommonProps> & {
    href?: undefined;
    external?: undefined;
    asChild: true;
  };

type ButtonProps = ButtonAsButton | ButtonAsLink | ButtonAsChild;

function isExternalUrl(url: string): boolean {
  return (
    url.startsWith("http://") ||
    url.startsWith("https://") ||
    url.startsWith("//")
  );
}

function Button(props: ButtonProps) {
  const {
    className,
    variant,
    size,
    loading = false,
    disabled = false,
    children,
    ...rest
  } = props;

  const isDisabled = loading || disabled;
  const buttonClasses = cn(buttonVariants({ variant, size, className }));

  if ("href" in rest && rest.href !== undefined) {
    const { href, external, ...linkProps } = rest;
    const isExternal = external ?? isExternalUrl(href);

    const sharedLinkProps = {
      className: buttonClasses,
      ...linkProps,
      ...(isDisabled && {
        "aria-disabled": true as const,
        tabIndex: -1,
        onClick: (e: React.MouseEvent) => e.preventDefault(),
      }),
    };

    if (isExternal) {
      return (
        <a
          href={href}
          target="_blank"
          rel="noopener noreferrer"
          {...sharedLinkProps}
        >
          {children}
        </a>
      );
    }

    return (
      <Link href={href} {...sharedLinkProps}>
        {children}
      </Link>
    );
  }

  if ("asChild" in rest && rest.asChild === true) {
    const { asChild, ...slotProps } = rest;
    return (
      <Slot
        className={buttonClasses}
        {...(isDisabled && { "aria-disabled": true, tabIndex: -1 })}
        {...slotProps}
      >
        {children}
      </Slot>
    );
  }

  const { asChild, external, ...buttonProps } = rest;
  return (
    <button
      type={"type" in buttonProps ? buttonProps.type : "button"}
      disabled={isDisabled}
      className={cn(
        buttonClasses,
        "relative",
        loading && "[&>*:not(.button-spinner)]:opacity-0"
      )}
      {...buttonProps}
    >
      {loading && (
        <div className="button-spinner absolute inset-0 flex items-center justify-center bg-inherit rounded-[inherit]">
          <Spinner />
        </div>
      )}
      {children}
    </button>
  );
}

export { Button, buttonVariants };
export type { ButtonProps };
