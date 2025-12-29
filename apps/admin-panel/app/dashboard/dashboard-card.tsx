"use client"

import Link from "next/link"
import { HiArrowRight } from "react-icons/hi"
import { ReactNode } from "react"

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"

type DashboardCardProps = {
  h1?: ReactNode
  h2?: ReactNode
  h2PopupDescription?: string
  title: string
  description: string
  to?: string
  buttonToRight?: boolean
  content?: React.ReactElement
  buttonText?: string
}

const DashboardCard: React.FC<DashboardCardProps> = ({
  h1,
  h2,
  h2PopupDescription,
  title,
  description,
  to = "",
  content,
  buttonToRight = false,
  buttonText = "",
}) => {
  return (
    <Card
      className="w-full flex flex-col justify-between"
      data-testid={title.toLowerCase().replace(" ", "-")}
    >
      <CardHeader>
        <div className="flex flex-col">
          {h2 ? (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="text-md text-muted-foreground min-h-[27px]">{h2}</span>
                </TooltipTrigger>
                {h2PopupDescription && (
                  <TooltipContent className="max-w-xs">
                    <p>{h2PopupDescription}</p>
                  </TooltipContent>
                )}
              </Tooltip>
            </TooltipProvider>
          ) : (
            <div className="h-[27px]" />
          )}
          {h1 && <CardTitle className="text-3xl font-bold">{h1}</CardTitle>}
        </div>
        <CardTitle className="text-lg font-medium">{title}</CardTitle>
        <CardDescription className="text-sm">{description}</CardDescription>
      </CardHeader>
      <CardContent>
        {content}
        {to && (
          <div className={`${buttonToRight ? "text-right" : ""}`}>
            <Button variant="outline" asChild>
              <Link href={to}>
                {buttonText}
                <HiArrowRight />
              </Link>
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default DashboardCard
