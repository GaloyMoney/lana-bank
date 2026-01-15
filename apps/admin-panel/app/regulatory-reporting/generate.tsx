"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { Button } from "@lana/web/ui/button"

import { ReportRunsDocument, useReportGenerateMutation } from "@/lib/graphql/generated"

gql`
  mutation ReportGenerate {
    triggerReportRun {
      runId
    }
  }
`

const ReportGeneration: React.FC = () => {
  const t = useTranslations("Reports.ReportGeneration")

  const [generateReport, { loading }] = useReportGenerateMutation({
    refetchQueries: [ReportRunsDocument],
  })

  const triggerGenerateReport = async () => {
    try {
      await generateReport()
      toast.success(t("reportGenerationHasBeenTriggered"))
    } catch {
      toast.error(t("reportGenerationFailed"))
    }
  }

  return (
    <Button type="button" disabled={loading} onClick={triggerGenerateReport}>
      {t("generate")}
    </Button>
  )
}

export { ReportGeneration }
