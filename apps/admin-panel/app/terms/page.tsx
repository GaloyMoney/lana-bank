"use client"
import React from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { gql } from "@apollo/client"

import { UpdateDefaultTermDialog } from "./update"

import { PageHeading } from "@/components/page-heading"
import { Button } from "@/components/primitive/button"
import { useDefaultTermsQuery } from "@/lib/graphql/generated"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatInterval, formatPeriod } from "@/lib/utils"
import { Separator } from "@/components/primitive/separator"

gql`
  query defaultTerms {
    defaultTerms {
      id
      termsId
      values {
        annualRate
        interval
        liquidationCvl
        marginCallCvl
        initialCvl
        duration {
          period
          units
        }
      }
    }
  }
`

function TermPage() {
  const { data, loading, error, refetch } = useDefaultTermsQuery()

  return (
    <main className="max-w-[70rem] m-auto">
      <PageHeading>Terms</PageHeading>
      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : data && data.defaultTerms ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center mb-0">
              <div className="flex flex-col space-y-1.5">
                <h2 className="font-semibold leading-none tracking-tight">
                  Default Terms
                </h2>
                <p className="text-textColor-secondary text-sm">
                  {data?.defaultTerms.termsId}
                </p>
              </div>
              <UpdateDefaultTermDialog refetch={refetch} termsData={data}>
                <Button variant="secondary" className="mt-6 flex gap-2 items-center">
                  <PiPencilSimpleLineLight className="w-5 h-5" />
                  Update Default Terms
                </Button>
              </UpdateDefaultTermDialog>
            </CardHeader>
            <Separator className="mb-4" />

            <CardContent>
              <DetailsGroup>
                <DetailItem
                  label="Duration"
                  value={
                    String(data.defaultTerms.values.duration.units) +
                    " " +
                    formatPeriod(data.defaultTerms.values.duration.period)
                  }
                />
                <DetailItem
                  label="Interval"
                  value={formatInterval(data.defaultTerms.values.interval)}
                />
                <DetailItem
                  label="Annual Rate"
                  value={data.defaultTerms.values.annualRate + "%"}
                />
                <DetailItem
                  label="Initial CVL"
                  value={data.defaultTerms.values.initialCvl + "%"}
                />
                <DetailItem
                  label="Margin Call CVL"
                  value={data.defaultTerms.values.marginCallCvl + "%"}
                />
                <DetailItem
                  label="Liquidation CVL"
                  value={data.defaultTerms.values.liquidationCvl + "%"}
                />
              </DetailsGroup>
            </CardContent>
          </>
        ) : (
          <div className="flex justify-between items-center">
            <CardContent className="pt-6">No data found</CardContent>
            <UpdateDefaultTermDialog refetch={refetch}>
              <Button variant="secondary" className="flex gap-2 items-center mr-4">
                <PiPencilSimpleLineLight className="w-5 h-5" />
                Update Default Terms
              </Button>
            </UpdateDefaultTermDialog>
          </div>
        )}
      </Card>
    </main>
  )
}

export default TermPage
