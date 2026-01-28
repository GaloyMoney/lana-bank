"use client"
import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import AuditLogsList from "./list"

const AuditLogs: React.FC = () => {
  const t = useTranslations("AuditLogs")

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <AuditLogsList page={1} />
        </CardContent>
      </Card>
    </>
  )
}

export default AuditLogs
