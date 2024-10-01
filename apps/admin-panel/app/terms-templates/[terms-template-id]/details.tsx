"use client"
import React from "react"

import { gql } from "@apollo/client"

import { DetailItem, DetailsGroup } from "@/components/details"
import { useTermsTemplateQuery } from "@/lib/graphql/generated"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { formatInterval, formatPeriod } from "@/lib/utils"
import { Separator } from "@/components/primitive/separator"

gql`
  query TermsTemplate($id: UUID!) {
    termsTemplate(id: $id) {
      id
      name
      termsId
      values {
        duration {
          units
          period
        }
        interval
        annualRate
        initialCvl
        marginCallCvl
        liquidationCvl
      }
    }
  }
`

function TermsTemplateDetails({ id }: { id: string }) {
  const { loading, error, data } = useTermsTemplateQuery({
    variables: { id },
  })

  return (
    <Card className="max-w-[70rem] m-auto">
      {loading ? (
        <CardContent className="pt-6">Loading...</CardContent>
      ) : error ? (
        <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
      ) : data && data.termsTemplate ? (
        <>
          <CardHeader className="flex flex-col justify-between">
            <CardTitle className="flex flex-col space-y-1.5">
              {data.termsTemplate.name}
            </CardTitle>
            <CardDescription className="text-sm text-textColor-secondary">
              {data.termsTemplate.termsId}
            </CardDescription>
          </CardHeader>
          <Separator className="mb-4" />

          <CardContent>
            <DetailsGroup>
              <DetailItem
                label="Duration"
                value={`${data.termsTemplate.values.duration.units} ${formatPeriod(data.termsTemplate.values.duration.period)}`}
              />
              <DetailItem
                label="Interval"
                value={formatInterval(data.termsTemplate.values.interval)}
              />
              <DetailItem
                label="Annual Rate"
                value={`${data.termsTemplate.values.annualRate}%`}
              />
              <DetailItem
                label="Initial CVL"
                value={`${data.termsTemplate.values.initialCvl}%`}
              />
              <DetailItem
                label="Margin Call CVL"
                value={`${data.termsTemplate.values.marginCallCvl}%`}
              />
              <DetailItem
                label="Liquidation CVL"
                value={`${data.termsTemplate.values.liquidationCvl}%`}
              />
            </DetailsGroup>
          </CardContent>
        </>
      ) : (
        <CardContent className="pt-6">No terms template found</CardContent>
      )}
    </Card>
  )
}

export default TermsTemplateDetails
