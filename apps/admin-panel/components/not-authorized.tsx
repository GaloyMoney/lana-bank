"use client"

import { useTranslations } from "next-intl"
import { ShieldX } from "lucide-react"

import { Card, CardContent } from "@lana/web/ui/card"

const NotAuthorized: React.FC = () => {
  const t = useTranslations("Common")

  return (
    <Card>
      <CardContent className="flex flex-col items-center justify-center py-12 text-center">
        <ShieldX className="h-12 w-12 text-muted-foreground mb-4" />
        <h3 className="text-lg font-semibold mb-2">{t("notAuthorized.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("notAuthorized.description")}</p>
      </CardContent>
    </Card>
  )
}

export default NotAuthorized
