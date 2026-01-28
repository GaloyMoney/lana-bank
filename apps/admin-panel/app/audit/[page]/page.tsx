"use client"
import { useTranslations } from "next-intl"
import { useParams, notFound } from "next/navigation"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import AuditLogsList from "../list"

const AuditLogsPage: React.FC = () => {
  const t = useTranslations("AuditLogs")
  const params = useParams()

  const pageParam = params.page as string
  const page = parseInt(pageParam, 10)

  if (isNaN(page) || page < 1) {
    notFound()
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <AuditLogsList page={page} />
      </CardContent>
    </Card>
  )
}

export default AuditLogsPage
