"use client"
import React, { useEffect, use } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"


import LedgerTransactions from "../../../components/ledger-transactions"

import { DisbursalDetailsCard } from "./details"
import { DisbursalEventHistory } from "./event-history"

import { VotersCard } from "./voters"

import { NotFound } from "@/components/not-found"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import {
  useGetDisbursalDetailsQuery,
  useDisbursalApprovalConcludedSubscription,
  DisbursalStatus,
} from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment DisbursalDetailsPageFragment on CreditFacilityDisbursal {
    id
    creditFacilityDisbursalId
    amount
    createdAt
    status
    publicId
    ledgerTransactions {
      ...LedgerTransactionFields
    }
    creditFacility {
      id
      creditFacilityId
      facilityAmount
      status
      publicId
      customer {
        id
        email
        customerId
        publicId
        depositAccount {
          id
          publicId
          balance {
            settled
            pending
          }
        }
      }
    }
    approvalProcess {
      ...ApprovalProcessFields
    }
  }

  query GetDisbursalDetails($publicId: PublicId!) {
    disbursalByPublicId(id: $publicId) {
      ...DisbursalDetailsPageFragment
    }
  }

  subscription disbursalApprovalConcluded($disbursalId: UUID!) {
    disbursalApprovalConcluded(disbursalId: $disbursalId) {
      status
      disbursal {
        ...DisbursalDetailsPageFragment
      }
    }
  }
`

function DisbursalPage({
  params,
}: {
  params: Promise<{
    "disbursal-id": string
  }>
}) {
  const { "disbursal-id": publicId } = use(params)
  const { data, loading, error } = useGetDisbursalDetailsQuery({
    variables: { publicId },
  })

  useDisbursalApprovalConcludedSubscription(
    data?.disbursalByPublicId &&
      data.disbursalByPublicId.status === DisbursalStatus.New
      ? { variables: { disbursalId: data.disbursalByPublicId.creditFacilityDisbursalId } }
      : { skip: true },
  )

  const { setDisbursal } = useCreateContext()
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")


  useEffect(() => {
    data?.disbursalByPublicId && setDisbursal(data.disbursalByPublicId)
    return () => setDisbursal(null)
  }, [data?.disbursalByPublicId, setDisbursal])

  useEffect(() => {
    if (data?.disbursalByPublicId) {
      setCustomLinks([
        { title: navTranslations("disbursals"), href: "/disbursals" },
        {
          title: <PublicIdBadge publicId={data.disbursalByPublicId.publicId} />,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.disbursalByPublicId])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} detailItems={5} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.disbursalByPublicId) return <NotFound />

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <DisbursalDetailsCard disbursal={data.disbursalByPublicId} />
      {data.disbursalByPublicId.approvalProcess && (
        <VotersCard approvalProcess={data.disbursalByPublicId.approvalProcess} />
      )}
      <LedgerTransactions
        ledgerTransactions={data.disbursalByPublicId.ledgerTransactions}
      />
      <DisbursalEventHistory publicId={publicId} />
    </main>
  )
}

export default DisbursalPage
