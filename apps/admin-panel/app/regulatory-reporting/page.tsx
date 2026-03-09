"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import { AvailableReports } from "./available-reports"
import { AvailableReportRuns } from "./list"

const RegulatorReportingPage: React.FC = () => {
  const t = useTranslations("Reports")

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col gap-1">
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <AvailableReportRuns />
        <AvailableReports />
      </CardContent>
    </Card>
  )
}

export default RegulatorReportingPage
