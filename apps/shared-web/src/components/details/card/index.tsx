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

const footerVariants = cva(
  "pt-4 pb-4 gap-4 w-full md:w-auto [&>*]:w-full md:[&>*]:w-auto md:[&>*]:mb-0 last:[&>*]:mb-0",
  {
    variants: {
      alignment: {
        left: "flex flex-col md:flex-row",
        right: "flex flex-col md:flex-row-reverse",
      },
    },
    defaultVariants: {
      alignment: "right",
    },
  },
)

export interface DetailsCardProps extends VariantProps<typeof footerVariants> {
  title: React.ReactNode
  description?: string
  details: DetailItemProps[]
  footerContent?: React.JSX.Element
  errorMessage?: string | undefined | null
  className?: string
  columns?: number
  layout?: "horizontal" | "vertical"
}

export const DetailsCard: React.FC<DetailsCardProps> = ({
  title,
  description,
  details,
  footerContent,
  errorMessage,
  alignment,
  className,
  columns,
  layout = "vertical",
}) => {
  const isBelowMedium = useBreakpointDown("md")
  const effectiveLayout = isBelowMedium ? "horizontal" : layout

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>
        <DetailsGroup columns={columns} layout={effectiveLayout}>
          {details.map((detail) => (
            <DetailItem
              key={detail.label?.toString()}
              {...detail}
              className={isBelowMedium ? "flex-1" : ""}
            />
          ))}
        </DetailsGroup>
      </CardContent>
      {errorMessage && (
        <CardFooter className="text-destructive">{errorMessage}</CardFooter>
      )}
      {footerContent && (
        <>
          <Separator />
          <CardFooter className={footerVariants({ alignment })}>
            {footerContent}
          </CardFooter>
        </>
      )}
    </Card>
  )
}
