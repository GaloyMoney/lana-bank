"use client"

import { useEffect, useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { Button } from "@lana/web/ui/button"
import { Badge } from "@lana/web/ui/badge"

import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import {
  ReportRunsDocument,
  ReportRunState,
  useReportGenerateMutation,
  useReportRunsQuery,
} from "@/lib/graphql/generated"

gql`
  mutation ReportGenerate {
    triggerReportRun
  }
`

const ReportGeneration: React.FC = () => {
  const t = useTranslations("Reports.ReportGeneration")

  const [triggered, setTriggered] = useState(false)
  const [generateReport, { loading: generateLoading }] = useReportGenerateMutation({
    refetchQueries: [ReportRunsDocument],
  })
  const { data, loading } = useReportRunsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const anyRunning =
    data?.reportRuns?.edges.some((edge) => edge.node.state === ReportRunState.Running) ||
    false

  const triggerGenerateReport = async () => {
    try {
      setTriggered(true)
      await generateReport()
      toast.success(t("reportGenerationHasBeenTriggered"))
    } catch {
      toast.error(t("reportGenerationFailed"))
    }
  }

  useEffect(() => {
    if (triggered && anyRunning) {
      setTriggered(false)
    }
  }, [anyRunning, triggered])

  if (loading || !data) return <Button disabled>{t("generate")}</Button>

  return (
    <>
      <div className="flex items-center gap-1 rounded-md">
        {anyRunning && (
          <Badge variant="secondary" className="p-2">
            <span className="inset-0 flex items-center justify-center">
              <span className="inline-block w-4 h-4 border-2 border-t-transparent border-current rounded-full animate-spin" />
            </span>
          </Badge>
        )}
        <Button
          type="button"
          disabled={anyRunning || generateLoading || triggered}
          onClick={triggerGenerateReport}
        >
          {t("generate")}
        </Button>
      </div>
    </>
  )
}

export { ReportGeneration }
