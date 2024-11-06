"use client"

import React from "react"

import { gql } from "@apollo/client"

import {
  useGetKycStatusForCustomerQuery,
  useSumsubPermalinkCreateMutation,
} from "@/lib/graphql/generated"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { DetailItem } from "@/components/details"
import { Skeleton } from "@/components/primitive/skeleton"

gql`
  query GetKycStatusForCustomer($id: UUID!) {
    customer(id: $id) {
      customerId
      status
      level
      applicantId
    }
  }

  mutation sumsubPermalinkCreate($input: SumsubPermalinkCreateInput!) {
    sumsubPermalinkCreate(input: $input) {
      url
    }
  }
`

type KycStatusProps = {
  customerId: string
}

export const KycStatus: React.FC<KycStatusProps> = ({ customerId }) => {
  const { data, loading } = useGetKycStatusForCustomerQuery({
    variables: {
      id: customerId,
    },
  })

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${data?.customer?.applicantId}/client/basicInfo`

  const [createLink, { data: linkData, loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation()

  const handleCreateLink = async () => {
    if (data?.customer?.customerId) {
      await createLink({
        variables: {
          input: {
            customerId: data.customer.customerId,
          },
        },
      })
    }
  }

  if (loading) return <Skeleton />

  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>KYC Status</CardTitle>
      </CardHeader>
      <CardContent>
        <DetailItem label="Level" value={data?.customer?.level.toLocaleLowerCase()} />
        <DetailItem label="Status" value={data?.customer?.status.toLocaleLowerCase()} />
        <DetailItem
          label="KYC Application Link"
          value={
            data?.customer?.applicantId ? (
              <a
                href={sumsubLink}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 underline"
              >
                {data?.customer.applicantId}
              </a>
            ) : (
              <div>
                {!linkData && (
                  <button
                    onClick={handleCreateLink}
                    className="text-blue-500 underline"
                    disabled={linkLoading}
                  >
                    {linkLoading ? "creating link..." : "Create link"}
                  </button>
                )}
                {linkData && linkData.sumsubPermalinkCreate && (
                  <a
                    href={linkData.sumsubPermalinkCreate.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-500 underline"
                  >
                    {linkData.sumsubPermalinkCreate.url}
                  </a>
                )}
                {linkError && <p className="text-red-500">{linkError.message}</p>}
              </div>
            )
          }
        />
      </CardContent>
    </Card>
  )
}
