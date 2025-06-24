"use client"

import React, { useState, useEffect, useRef } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"
import { Loader2, FileDown, FileUp, CheckCircle, AlertCircle, Clock } from "lucide-react"

import { formatDate } from "@lana/web/utils"

import {
  useAccountingCsvsForLedgerAccountIdQuery,
  useLedgerAccountCsvCreateMutation,
  useAccountingCsvDownloadLinkGenerateMutation,
  AccountingCsvStatus,
} from "@/lib/graphql/generated"

gql`
  query AccountingCsvsForLedgerAccountId(
    $ledgerAccountId: UUID!
    $first: Int!
    $after: String
  ) {
    accountingCsvsForLedgerAccountId(
      ledgerAccountId: $ledgerAccountId
      first: $first
      after: $after
    ) {
      edges {
        node {
          id
          documentId
          status
          createdAt
        }
        cursor
      }
      pageInfo {
        hasNextPage
        endCursor
      }
    }
  }

  mutation LedgerAccountCsvCreate($input: LedgerAccountCsvCreateInput!) {
    ledgerAccountCsvCreate(input: $input) {
      accountingCsvDocument {
        id
        status
        createdAt
      }
    }
  }

  mutation AccountingCsvDownloadLinkGenerate(
    $input: AccountingCsvDownloadLinkGenerateInput!
  ) {
    accountingCsvDownloadLinkGenerate(input: $input) {
      link {
        url
        csvId
      }
    }
  }
`

type CsvOption = {
  id: string
  documentId: string
  label: string
  status: AccountingCsvStatus
  createdAt: string
}

type ExportCsvDialogProps = {
  isOpen: boolean
  onClose: () => void
  ledgerAccountId: string
}

export const ExportCsvDialog: React.FC<ExportCsvDialogProps> = ({
  isOpen,
  onClose,
  ledgerAccountId,
}) => {
  const t = useTranslations("ChartOfAccountsLedgerAccount.exportCsv")
  const [selectedCsvId, setSelectedCsvId] = useState<string | null>(null)
  const [csvOptions, setCsvOptions] = useState<CsvOption[]>([])
  const [isDownloading, setIsDownloading] = useState(false)

  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const pollingCsvIdRef = useRef<string | null>(null)

  const { data, loading, error, refetch } = useAccountingCsvsForLedgerAccountIdQuery({
    variables: {
      ledgerAccountId,
      first: 5,
    },
    skip: !isOpen,
    fetchPolicy: "network-only",
    notifyOnNetworkStatusChange: false,
  })

  const [createCsv, { loading: createLoading }] = useLedgerAccountCsvCreateMutation()
  const [generateDownloadLink] = useAccountingCsvDownloadLinkGenerateMutation()

  useEffect(() => {
    if (data?.accountingCsvsForLedgerAccountId.edges) {
      const options = data.accountingCsvsForLedgerAccountId.edges.map((edge) => ({
        id: edge.node.id,
        documentId: edge.node.documentId,
        label: `${formatDate(edge.node.createdAt)} - ${t(`status.${edge.node.status.toLowerCase()}`)}`,
        status: edge.node.status,
        createdAt: edge.node.createdAt,
      }))
      setCsvOptions(options)
      if (pollingCsvIdRef.current) {
        const pollingCsv = options.find((opt) => opt.documentId === pollingCsvIdRef.current)
        if (pollingCsv) {
          if (selectedCsvId !== pollingCsvIdRef.current) {
            setSelectedCsvId(pollingCsvIdRef.current)
          }
          if (pollingCsv.status === AccountingCsvStatus.Completed) {
            stopPolling()
            toast.success(t("csvReady"))
          } else if (pollingCsv.status === AccountingCsvStatus.Failed) {
            stopPolling()
            toast.error(t("csvFailed"))
          }
        }
      } else if (!selectedCsvId && options.length > 0) {
        setSelectedCsvId(options[0].documentId)
      }
    }
  }, [data, selectedCsvId, t])

  const startPolling = (csvId: string) => {
    pollingCsvIdRef.current = csvId
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
    }
    pollingIntervalRef.current = setInterval(() => {
      refetch().catch(() => stopPolling())
    }, 2000)
  }

  const stopPolling = () => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
      pollingIntervalRef.current = null
    }
    pollingCsvIdRef.current = null
  }

  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
    }
  }, [])

  const handleCreateNewCsv = async () => {
    try {
      const result = await createCsv({
        variables: {
          input: {
            ledgerAccountId,
          },
        },
      })

      if (result.data) {
        const newCsvId = result.data.ledgerAccountCsvCreate.accountingCsvDocument.documentId
        toast.success(t("csvCreating"))
        startPolling(newCsvId)
        await refetch()
      }
    } catch (err) {
      console.error("Error creating CSV:", err)
      toast.error(t("errors.createFailed"))
    }
  }

  const handleDownload = async () => {
    if (!selectedCsvId) return

    const selectedOption = csvOptions.find((opt) => opt.documentId === selectedCsvId)
    if (!selectedOption || selectedOption.status !== AccountingCsvStatus.Completed) {
      toast.error(t("errors.notReady"))
      return
    }

    try {
      setIsDownloading(true)
      const result = await generateDownloadLink({
        variables: {
          input: {
            documentId: selectedCsvId,
          },
        },
      })

      if (result.data?.accountingCsvDownloadLinkGenerate.link.url) {
        const url = result.data.accountingCsvDownloadLinkGenerate.link.url
        window.open(url, "_blank")
      }
    } catch (err) {
      console.error("Error downloading:", err)
      toast.error(t("errors.downloadFailed"))
    } finally {
      setIsDownloading(false)
    }
  }

  const handleSelectChange = (value: string) => {
    setSelectedCsvId(value)
  }

  const handleClose = () => {
    stopPolling()
    setSelectedCsvId(null)
    onClose()
  }

  const isSelectedCompleted = selectedCsvId
    ? csvOptions.find((opt) => opt.documentId === selectedCsvId)?.status ===
      AccountingCsvStatus.Completed
    : false

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          <div>
            <h3 className="text-sm font-medium mb-3">{t("existingExports")}</h3>
            {loading ? (
              <div className="flex justify-center py-4">
                <Loader2 className="h-6 w-6 animate-spin text-primary" />
              </div>
            ) : error ? (
              <div className="text-destructive p-2 text-center text-sm">
                {t("errors.loadFailed")}
              </div>
            ) : csvOptions.length === 0 ? (
              <div className="text-center py-2 text-muted-foreground text-sm">
                {t("noCsvs")}
              </div>
            ) : (
              <div className="space-y-4">
                <Select value={selectedCsvId || ""} onValueChange={handleSelectChange}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("selectExport")} />
                  </SelectTrigger>
                  <SelectContent>
                    {csvOptions.map((option) => (
                      <SelectItem key={option.id} value={option.documentId}>
                        <div className="flex items-center">
                          {option.status === AccountingCsvStatus.Completed ? (
                            <CheckCircle className="h-4 w-4 mr-2 text-success" />
                          ) : option.status === AccountingCsvStatus.Pending ? (
                            <Clock className="h-4 w-4 mr-2 text-warning" />
                          ) : (
                            <AlertCircle className="h-4 w-4 mr-2 text-destructive" />
                          )}
                          {formatDate(option.createdAt)}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Button
                  className="w-full"
                  onClick={handleDownload}
                  disabled={!selectedCsvId || !isSelectedCompleted || isDownloading}
                >
                  {isDownloading ? (
                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  ) : (
                    <FileDown className="h-4 w-4 mr-2" />
                  )}
                  {t("buttons.download")}
                </Button>

                {pollingCsvIdRef.current && (
                  <div className="flex items-center justify-center py-2 text-muted-foreground text-sm">
                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                    {t("processing")}
                  </div>
                )}
              </div>
            )}
          </div>

          <div className="border-t pt-4">
            <h3 className="text-sm font-medium mb-3">{t("createNew")}</h3>
            <p className="text-sm text-muted-foreground mb-4">
              {t("createNewDescription")}
            </p>
            <Button
              onClick={handleCreateNewCsv}
              disabled={createLoading}
              className="w-full"
            >
              {createLoading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  {t("buttons.generating")}
                </>
              ) : (
                <>
                  <FileUp className="h-4 w-4 mr-2" />
                  {t("buttons.generate")}
                </>
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
