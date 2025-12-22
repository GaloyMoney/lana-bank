"use client"
import React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Separator } from "@lana/web/ui/separator"
import { useBreakpointDown } from "@lana/web/hooks"

import { DetailItem, DetailItemProps, DetailsGroup } from ".."

import { cn } from "@/lib/utils"
import { PublicIdBadge } from "@/components/public-id-badge"

const footerVariants = cva("pt-0 pb-0 gap-4 w-full", {
  variants: {
    alignment: {
      left: "flex flex-col md:flex-row",
      right: "flex flex-col md:flex-row-reverse",
    },
  },
  defaultVariants: {
    alignment: "right",
  },
})

const containerVariants = cva("pb-4", {
  variants: {
    variant: {
      card: "",
      container: "",
    },
  },
  defaultVariants: {
    variant: "card",
  },
})

export interface DetailsCardProps
  extends VariantProps<typeof footerVariants>,
    VariantProps<typeof containerVariants> {
  title?: string
  badge?: React.JSX.Element
  description?: string
  details: DetailItemProps[]
  footerContent?: React.JSX.Element | null
  headerAction?: React.JSX.Element
  errorMessage?: string | undefined | null
  className?: string
  columns?: number
  layout?: "horizontal" | "vertical"
  footerClassName?: string
  publicId?: string
}

export const DetailsCard: React.FC<DetailsCardProps> = ({
  title,
  badge,
  description,
  details,
  footerContent,
  headerAction,
  errorMessage,
  alignment,
  className,
  columns,
  layout = "vertical",
  variant = "card",
  footerClassName,
  publicId,
}) => {
  const isBelowMedium = useBreakpointDown("md")
  const effectiveLayout = isBelowMedium ? "horizontal" : layout

  const content = (
    <>
      {variant === "container" ? (
        <div className="flex items-center justify-between">
          <div className="flex flex-col space-y-2">
            <div className={cn("font-semibold leading-none tracking-tight")}>{title}</div>
            {description && (
              <div className={cn("text-sm text-muted-foreground")}>{description}</div>
            )}
          </div>
          {headerAction && <div>{headerAction}</div>}
        </div>
      ) : (
        <div>
          <div className={cn("font-semibold leading-none tracking-tight")}>{title}</div>
          {description && (
            <div className={cn("text-sm text-muted-foreground")}>{description}</div>
          )}
        </div>
      )}
      <div>
        <DetailsGroup columns={columns} layout={effectiveLayout}>
          {details.map((detail, idx) => {
            const { className: detailClassName, ...detailProps } = detail
            return (
              <DetailItem
                key={`${detailProps.label?.toString()}-${idx}`}
                {...detailProps}
                className={cn(detailClassName, isBelowMedium ? "flex-1" : "")}
              />
            )
          })}
        </DetailsGroup>
      </div>
      {errorMessage && <div className="text-destructive">{errorMessage}</div>}
      {footerContent && (
        <>
          {variant === "card" && <Separator />}
          <div className={footerVariants({ alignment })}>{footerContent}</div>
        </>
      )}
    </>
  )

  if (variant === "container") {
    return <div className={cn(containerVariants({ variant }), className)}>{content}</div>
  }

  return (
    <Card className={cn(containerVariants({ variant }), className)}>
      <CardHeader>
        <div className="flex items-center justify-between w-full">
          <div className="space-y-1.5">
            <div className="flex items-center gap-2.5">
              <CardTitle>{title}</CardTitle>
              {publicId && (
                <div className={cn("text-sm text-muted-foreground")}>
                  <PublicIdBadge publicId={publicId} />
                </div>
              )}
              {badge}
            </div>
            {description && <CardDescription>{description}</CardDescription>}
          </div>
          {headerAction && <div>{headerAction}</div>}
        </div>
      </CardHeader>
      <CardContent>
        <DetailsGroup columns={columns} layout={effectiveLayout}>
          {details.map((detail, idx) => {
            const { className: detailClassName, ...detailProps } = detail
            return (
              <DetailItem
                key={`${detailProps.label?.toString()}-${idx}`}
                {...detailProps}
                className={cn(detailClassName, isBelowMedium ? "flex-1" : "")}
              />
            )
          })}
        </DetailsGroup>
      </CardContent>
      {errorMessage && (
        <CardFooter className="text-destructive">{errorMessage}</CardFooter>
      )}
      {footerContent && (
        <>
          <Separator className="-mb-2" />
          <CardFooter className={cn(footerVariants({ alignment }), footerClassName)}>
            {footerContent}
          </CardFooter>
        </>
      )}
    </Card>
  )
}
