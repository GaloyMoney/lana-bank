import { useState, useRef, useCallback, useEffect } from "react"
import { toast } from "sonner"

import { useTranslations } from "next-intl"

import { gql } from "@apollo/client"

import {
  useCreditFacilityAgreementGenerateMutation,
  useCreditFacilityAgreementDownloadLinkGenerateMutation,
  useCreditFacilityAgreementLazyQuery,
  CreditFacilityAgreementStatus,
} from "@/lib/graphql/generated"

gql`
  mutation CreditFacilityAgreementGenerate($input: CreditFacilityAgreementGenerateInput!) {
    creditFacilityAgreementGenerate(input: $input) {
      creditFacilityAgreement {
        creditFacilityAgreementId
        status
        createdAt
      }
    }
  }

  mutation CreditFacilityAgreementDownloadLinkGenerate(
    $input: CreditFacilityAgreementDownloadLinksGenerateInput!
  ) {
    creditFacilityAgreementDownloadLinkGenerate(input: $input) {
      creditFacilityAgreementId
      link
    }
  }

  query CreditFacilityAgreement($id: UUID!) {
    creditFacilityAgreement(id: $id) {
      creditFacilityAgreementId
      status
      createdAt
    }
  }
`

export const useCreditFacilityAgreement = () => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.DetailsCard")
  const [isGenerating, setIsGenerating] = useState(false)

  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const pollingAgreementIdRef = useRef<string | null>(null)

  const [generateAgreement] = useCreditFacilityAgreementGenerateMutation()
  const [generateDownloadLink] = useCreditFacilityAgreementDownloadLinkGenerateMutation()
  const [getAgreement] = useCreditFacilityAgreementLazyQuery({
    fetchPolicy: "network-only",
  })

  const handleError = useCallback(
    (error?: unknown) => {
      console.error("Error generating credit facility agreement:", error)
      toast.error(t("creditFacilityAgreement.error"))
      setIsGenerating(false)
    },
    [t],
  )

  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
      pollingIntervalRef.current = null
    }
    pollingAgreementIdRef.current = null
  }, [])

  const handleDownload = useCallback(
    async (creditFacilityAgreementId: string) => {
      try {
        const linkResult = await generateDownloadLink({
          variables: {
            input: {
              creditFacilityAgreementId,
            },
          },
        })

        const downloadLink =
          linkResult.data?.creditFacilityAgreementDownloadLinkGenerate?.link
        if (downloadLink) {
          window.open(downloadLink, "_blank")
          toast.success(t("creditFacilityAgreement.success"))
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
    (creditFacilityAgreementId: string) => {
      pollingAgreementIdRef.current = creditFacilityAgreementId
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
      pollingIntervalRef.current = setInterval(async () => {
        try {
          const result = await getAgreement({
            variables: { id: creditFacilityAgreementId },
          })

          const status = result.data?.creditFacilityAgreement?.status
          if (status === CreditFacilityAgreementStatus.Completed) {
            stopPolling()
            await handleDownload(creditFacilityAgreementId)
          } else if (status === CreditFacilityAgreementStatus.Failed) {
            stopPolling()
            handleError()
          }
        } catch (error) {
          stopPolling()
          handleError(error)
        }
      }, 2000)
    },
    [getAgreement, stopPolling, handleError, handleDownload],
  )

  const generateCreditFacilityAgreementPdf = useCallback(
    async (customerId: string) => {
      setIsGenerating(true)
      try {
        const generateResult = await generateAgreement({
          variables: {
            input: {
              customerId,
            },
          },
        })

        const agreement =
          generateResult.data?.creditFacilityAgreementGenerate?.creditFacilityAgreement
        if (!agreement) {
          throw new Error("Failed to generate credit facility agreement")
        }

        if (agreement.status === CreditFacilityAgreementStatus.Completed) {
          await handleDownload(agreement.creditFacilityAgreementId)
        } else if (agreement.status === CreditFacilityAgreementStatus.Pending) {
          startPolling(agreement.creditFacilityAgreementId)
        } else {
          throw new Error("Unexpected credit facility agreement status")
        }
      } catch (error) {
        handleError(error)
      }
    },
    [generateAgreement, startPolling, handleError, handleDownload],
  )

  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
      }
    }
  }, [])

  return {
    generateCreditFacilityAgreementPdf,
    isGenerating,
  }
}
