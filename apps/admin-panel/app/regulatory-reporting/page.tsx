"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import { ReportGeneration } from "./generate"
import { AvailableReportRuns } from "./list"

const RegulatorReportingPage: React.FC = () => {
  const t = useTranslations("Reports")

  return (
    <Card>
      <CardHeader className="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div className="flex flex-col gap-1">
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </div>
        <ReportGeneration />
      </CardHeader>
      <CardContent>
        <AvailableReportRuns />
      </CardContent>
    </Card>
  )
}

export default RegulatorReportingPage
