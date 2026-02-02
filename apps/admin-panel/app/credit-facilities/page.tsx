"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import { Button } from "@lana/web/ui/button"
import { Download } from "lucide-react"

import CustomersList from "./list"
import { usePdfGenerate } from "@/hooks/use-pdf-generate"

const CreditFacilities: React.FC = () => {
  const t = useTranslations("CreditFacilities")
  const { generate, isGenerating } = usePdfGenerate()

  const handleExport = () => {
    generate(
      {
        creditFacilityExport: {
          generate: true,
        },
      },
      {
        successMessage: t("export.success"),
        errorMessage: t("export.error"),
      }
    )
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{t("title")}</CardTitle>
            <CardDescription>{t("description")}</CardDescription>
          </div>
          <Button
            variant="outline"
            onClick={handleExport}
            loading={isGenerating}
            data-testid="export-credit-facilities-button"
          >
            <Download className="h-4 w-4 mr-2" />
            {t("export.button")}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <CustomersList />
      </CardContent>
    </Card>
  )
}

export default CreditFacilities
