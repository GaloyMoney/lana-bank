"use client"

import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@lana/web/utils";

const detailsGroupVariants = cva("", {
  variants: {
    layout: {
      vertical: "grid gap-6",
      horizontal: "flex flex-col",
    },
  },
  defaultVariants: {
    layout: "vertical",
  },
})

type LayoutType = NonNullable<VariantProps<typeof detailsGroupVariants>["layout"]>

interface DetailsGroupProps
  extends Omit<VariantProps<typeof detailsGroupVariants>, "layout"> {
  children: React.ReactNode
  className?: string
  layout?: LayoutType
  columns?: number
}

export const DetailsGroupContext = React.createContext<LayoutType>("vertical")

export const DetailsGroup: React.FC<DetailsGroupProps> = ({
  children,
  layout = "vertical",
  className,
  columns,
}) => {
  const childrenArray = React.Children.toArray(children)
  const gridColumns = {
    1: "grid-cols-1",
    2: "grid-cols-2",
    3: "grid-cols-3",
    4: "grid-cols-4",
  }

  return (
    <DetailsGroupContext.Provider value={layout}>
      <div
        className={cn(
          detailsGroupVariants({ layout }),
          layout === "vertical" &&
            (columns
              ? gridColumns[columns as keyof typeof gridColumns]
              : childrenArray.length > 2
                ? "grid-cols-4"
                : "grid-cols-2"),
          className,
        )}
      >
        {childrenArray}
      </div>
    </DetailsGroupContext.Provider>
  )
}
