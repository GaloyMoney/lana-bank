"use client"

import React, { useState, useEffect } from "react"
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
import { Loader2, FileDown, FileUp } from "lucide-react"

import { formatDate } from "@lana/web/utils"

import {
  AccountEntryCsvDocument,
  useAccountEntryCsvQuery,
  useLedgerAccountCsvCreateMutation,
  useAccountingCsvDownloadLinkGenerateMutation,
  useLedgerAccountCsvExportUploadedSubscription,
  DocumentStatus,
} from "@/lib/graphql/generated"

gql`
  query AccountEntryCsv($ledgerAccountId: UUID!) {
    accountEntryCsv(ledgerAccountId: $ledgerAccountId) {
      id
      documentId
      status
      createdAt
    }
  }

  mutation LedgerAccountCsvCreate($input: LedgerAccountCsvCreateInput!) {
    ledgerAccountCsvCreate(input: $input) {
      accountingCsvDocument {
        id
        documentId
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

  subscription LedgerAccountCsvExportUploaded($ledgerAccountId: UUID!) {
    ledgerAccountCsvExportUploaded(ledgerAccountId: $ledgerAccountId) {
      documentId
    }
  }
`

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
  const [isDownloading, setIsDownloading] = useState(false)

  const { data, loading, error, refetch } = useAccountEntryCsvQuery({
    variables: { ledgerAccountId },
    skip: !isOpen,
    fetchPolicy: "network-only",
    notifyOnNetworkStatusChange: false,
  })

  const { data: subscriptionData } = useLedgerAccountCsvExportUploadedSubscription({
    variables: { ledgerAccountId },
    skip: !isOpen,
  })

  const [createCsv, { loading: createLoading }] = useLedgerAccountCsvCreateMutation({
    update: (cache, { data }) => {
      const created = data?.ledgerAccountCsvCreate?.accountingCsvDocument
      if (!created) return

      cache.writeQuery({
        query: AccountEntryCsvDocument,
        variables: { ledgerAccountId },
        data: { accountEntryCsv: created },
      })
    },
  })
  const [generateDownloadLink] = useAccountingCsvDownloadLinkGenerateMutation()

  useEffect(() => {
    if (!subscriptionData?.ledgerAccountCsvExportUploaded) {
      return
    }
    refetch()
  }, [subscriptionData, refetch])

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
        toast.success(t("csvCreating"))
      }
    } catch (err) {
      console.error("Error creating CSV:", err)
      toast.error(t("errors.createFailed"))
    }
  }

  const handleDownload = async () => {
    const currentCsv = data?.accountEntryCsv
    if (!currentCsv) return
    if (currentCsv.status !== DocumentStatus.Active) {
      toast.error(t("errors.notReady"))
      return
    }

    try {
      setIsDownloading(true)
      const result = await generateDownloadLink({
        variables: {
          input: {
            documentId: currentCsv.documentId,
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

  const handleClose = () => {
    onClose()
  }

  const currentCsv = data?.accountEntryCsv ?? null
  const isCurrentCompleted = currentCsv?.status === DocumentStatus.Active

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
            ) : !currentCsv ? (
              <div className="text-center py-2 text-muted-foreground text-sm">
                {t("noCsvs")}
              </div>
            ) : (
              <div className="space-y-4">
                <div className="rounded-md border px-3 py-2 text-sm">
                  <div className="flex items-center justify-between">
                    <span>{formatDate(currentCsv.createdAt)}</span>
                    <span className="text-muted-foreground">
                      {t(`status.${currentCsv.status.toLowerCase()}`)}
                    </span>
                  </div>
                </div>

                <Button
                  className="w-full"
                  onClick={handleDownload}
                  disabled={!isCurrentCompleted || isDownloading}
                >
                  {isDownloading ? (
                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  ) : (
                    <FileDown className="h-4 w-4 mr-2" />
                  )}
                  {t("buttons.download")}
                </Button>

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
