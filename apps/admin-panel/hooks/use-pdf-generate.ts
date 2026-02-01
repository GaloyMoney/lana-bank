import { useState, useRef, useCallback, useEffect } from "react"
import { toast } from "sonner"

import { gql } from "@apollo/client"

import {
  usePdfGenerateMutation,
  usePdfDownloadLinkGenerateMutation,
  useLoanAgreementLazyQuery,
  useCreditFacilityExportLazyQuery,
  LoanAgreementStatus,
  CreditFacilityExportStatus,
  PdfGenerateInput,
} from "@/lib/graphql/generated"

gql`
  mutation PdfGenerate($input: PdfGenerateInput!) {
    pdfGenerate(input: $input) {
      document {
        ... on LoanAgreement {
          id
          loanAgreementStatus: status
          createdAt
        }
        ... on CreditFacilityExport {
          id
          creditFacilityExportStatus: status
          createdAt
        }
      }
    }
  }

  mutation PdfDownloadLinkGenerate($input: PdfDownloadLinkGenerateInput!) {
    pdfDownloadLinkGenerate(input: $input) {
      pdfId
      link
    }
  }

  query LoanAgreement($id: UUID!) {
    loanAgreement(id: $id) {
      id
      status
      createdAt
    }
  }

  query CreditFacilityExport($id: UUID!) {
    creditFacilityExport(id: $id) {
      id
      status
      createdAt
    }
  }
`

type DocumentStatus = LoanAgreementStatus | CreditFacilityExportStatus

const COMPLETED_STATUSES = [
  LoanAgreementStatus.Completed,
  CreditFacilityExportStatus.Completed,
]

const PENDING_STATUSES = [
  LoanAgreementStatus.Pending,
  CreditFacilityExportStatus.Pending,
]

const FAILED_STATUSES = [
  LoanAgreementStatus.Failed,
  CreditFacilityExportStatus.Failed,
]

export const usePdfGenerate = () => {
  const [isGenerating, setIsGenerating] = useState(false)

  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const pollingDocumentIdRef = useRef<string | null>(null)
  const documentTypeRef = useRef<string | null>(null)

  const [generatePdf] = usePdfGenerateMutation()
  const [generateDownloadLink] = usePdfDownloadLinkGenerateMutation()
  const [getLoanAgreement] = useLoanAgreementLazyQuery({
    fetchPolicy: "network-only",
  })
  const [getCreditFacilityExport] = useCreditFacilityExportLazyQuery({
    fetchPolicy: "network-only",
  })

  const handleError = useCallback((error?: unknown, errorMessage?: string) => {
    console.error("Error generating PDF:", error)
    toast.error(errorMessage || "Failed to generate PDF")
    setIsGenerating(false)
  }, [])

  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
      pollingIntervalRef.current = null
    }
    pollingDocumentIdRef.current = null
    documentTypeRef.current = null
  }, [])

  const handleDownload = useCallback(
    async (pdfId: string, successMessage?: string) => {
      try {
        const linkResult = await generateDownloadLink({
          variables: {
            input: {
              pdfId,
            },
          },
        })

        const downloadLink = linkResult.data?.pdfDownloadLinkGenerate?.link
        if (downloadLink) {
          window.open(downloadLink, "_blank")
          toast.success(successMessage || "PDF generated successfully")
        } else {
          throw new Error("Failed to generate download link")
        }
      } catch (error) {
        handleError(error)
      } finally {
        setIsGenerating(false)
      }
    },
    [generateDownloadLink, handleError],
  )

  const getDocumentStatus = useCallback(
    async (documentId: string, documentType: string): Promise<DocumentStatus | null> => {
      if (documentType === "LoanAgreement") {
        const result = await getLoanAgreement({
          variables: { id: documentId },
        })
        return result.data?.loanAgreement?.status ?? null
      } else if (documentType === "CreditFacilityExport") {
        const result = await getCreditFacilityExport({
          variables: { id: documentId },
        })
        return result.data?.creditFacilityExport?.status ?? null
      }
      return null
    },
    [getLoanAgreement, getCreditFacilityExport],
  )

  const startPolling = useCallback(
    (documentId: string, documentType: string, successMessage?: string, errorMessage?: string) => {
      pollingDocumentIdRef.current = documentId
      documentTypeRef.current = documentType
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
      pollingIntervalRef.current = setInterval(async () => {
        try {
          const status = await getDocumentStatus(documentId, documentType)

          if (COMPLETED_STATUSES.includes(status as DocumentStatus)) {
            stopPolling()
            await handleDownload(documentId, successMessage)
          } else if (FAILED_STATUSES.includes(status as DocumentStatus)) {
            stopPolling()
            handleError(undefined, errorMessage)
          }
        } catch (error) {
          stopPolling()
          handleError(error, errorMessage)
        }
      }, 2000)
    },
    [getDocumentStatus, stopPolling, handleError, handleDownload],
  )

  const generate = useCallback(
    async (
      input: PdfGenerateInput,
      options?: {
        successMessage?: string
        errorMessage?: string
      }
    ) => {
      setIsGenerating(true)
      try {
        const generateResult = await generatePdf({
          variables: {
            input,
          },
        })

        const document = generateResult.data?.pdfGenerate?.document
        if (!document) {
          throw new Error("Failed to generate PDF")
        }

        const { __typename: documentType, id } = document

        // Get status from the correct field based on document type
        let status: DocumentStatus
        if (documentType === "LoanAgreement" && "loanAgreementStatus" in document) {
          status = document.loanAgreementStatus
        } else if (documentType === "CreditFacilityExport" && "creditFacilityExportStatus" in document) {
          status = document.creditFacilityExportStatus
        } else {
          throw new Error("Unknown document type")
        }

        if (COMPLETED_STATUSES.includes(status)) {
          await handleDownload(id, options?.successMessage)
        } else if (PENDING_STATUSES.includes(status)) {
          // Poll for both LoanAgreement and CreditFacilityExport
          startPolling(id, documentType, options?.successMessage, options?.errorMessage)
        } else if (FAILED_STATUSES.includes(status)) {
          handleError(undefined, options?.errorMessage || "PDF generation failed")
        } else {
          throw new Error("Unexpected PDF status")
        }
      } catch (error) {
        handleError(error, options?.errorMessage)
      }
    },
    [generatePdf, startPolling, handleError, handleDownload],
  )

  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
    }
  }, [])

  return {
    generate,
    isGenerating,
  }
}
