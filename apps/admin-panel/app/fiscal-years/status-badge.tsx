import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

interface FiscalYearStatusBadgeProps {
  isOpen: boolean
}

export function FiscalYearStatusBadge({ isOpen }: FiscalYearStatusBadgeProps) {
  const t = useTranslations("FiscalYears.status")

  return (
    <Badge variant={isOpen ? "success" : "destructive"}>
      {isOpen ? t("open") : t("closed")}
    </Badge>
  )
}
