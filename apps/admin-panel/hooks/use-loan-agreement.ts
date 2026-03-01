import { useState, useRef, useCallback, useEffect } from "react"
import { toast } from "sonner"

import { useTranslations } from "next-intl"

import { gql } from "@apollo/client"

import {
  useLoanAgreementGenerateMutation,
  useLoanAgreementDownloadLinkGenerateMutation,
  useLoanAgreementLazyQuery,
  LoanAgreementStatus,
} from "@/lib/graphql/generated"

gql`
  mutation LoanAgreementGenerate($input: LoanAgreementGenerateInput!) {
    loanAgreementGenerate(input: $input) {
      loanAgreement {
        id
        loanAgreementId
        status
        createdAt
      }
    }
  }

  mutation LoanAgreementDownloadLinkGenerate(
    $input: LoanAgreementDownloadLinksGenerateInput!
  ) {
    loanAgreementDownloadLinkGenerate(input: $input) {
      loanAgreementId
      link
    }
  }

  query LoanAgreement($id: UUID!) {
    loanAgreement(id: $id) {
      id
      loanAgreementId
      status
      createdAt
    }
  }
`

export const useLoanAgreement = () => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.DetailsCard")
  const [isGenerating, setIsGenerating] = useState(false)

  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const pollingLoanAgreementIdRef = useRef<string | null>(null)

  const [generateLoanAgreement] = useLoanAgreementGenerateMutation()
  const [generateDownloadLink] = useLoanAgreementDownloadLinkGenerateMutation()
  const [getLoanAgreement] = useLoanAgreementLazyQuery({
    fetchPolicy: "network-only",
  })

  const handleError = useCallback(
    (error?: unknown) => {
      console.error("Error generating loan agreement:", error)
      toast.error(t("loanAgreement.error"))
      setIsGenerating(false)
    },
    [t],
  )

  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
      pollingIntervalRef.current = null
    }
    pollingLoanAgreementIdRef.current = null
  }, [])

  const handleDownload = useCallback(
    async (loanAgreementId: string) => {
      try {
        const linkResult = await generateDownloadLink({
          variables: {
            input: {
              loanAgreementId,
            },
          },
        })

        const downloadLink = linkResult.data?.loanAgreementDownloadLinkGenerate?.link
        if (downloadLink) {
          window.open(downloadLink, "_blank")
          toast.success(t("loanAgreement.success"))
        } else {
          throw new Error("Failed to generate download link")
        }
      } catch (error) {
        handleError(error)
      } finally {
        setIsGenerating(false)
      }
    },
    [generateDownloadLink, t, handleError],
  )

  const startPolling = useCallback(
    (loanAgreementId: string) => {
      pollingLoanAgreementIdRef.current = loanAgreementId
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
      pollingIntervalRef.current = setInterval(async () => {
        try {
          const result = await getLoanAgreement({
            variables: { id: loanAgreementId },
          })

          const status = result.data?.loanAgreement?.status
          if (status === LoanAgreementStatus.Completed) {
            stopPolling()
            await handleDownload(loanAgreementId)
          } else if (status === LoanAgreementStatus.Failed) {
            stopPolling()
            handleError()
          }
        } catch (error) {
          stopPolling()
          handleError(error)
        }
      }, 2000)
    },
    [getLoanAgreement, stopPolling, handleError, handleDownload],
  )

  const generateLoanAgreementPdf = useCallback(
    async (customerId: string) => {
      setIsGenerating(true)
      try {
        const generateResult = await generateLoanAgreement({
          variables: {
            input: {
              customerId,
            },
          },
        })

        const loanAgreement = generateResult.data?.loanAgreementGenerate?.loanAgreement
        if (!loanAgreement) {
          throw new Error("Failed to generate loan agreement")
        }

        if (loanAgreement.status === LoanAgreementStatus.Completed) {
          await handleDownload(loanAgreement.loanAgreementId)
        } else if (loanAgreement.status === LoanAgreementStatus.Pending) {
          startPolling(loanAgreement.loanAgreementId)
        } else {
          throw new Error("Unexpected loan agreement status")
        }
      } catch (error) {
        handleError(error)
      }
    },
    [generateLoanAgreement, startPolling, handleError, handleDownload],
  )

  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
    }
  }, [])

  return {
    generateLoanAgreementPdf,
    isGenerating,
  }
}
