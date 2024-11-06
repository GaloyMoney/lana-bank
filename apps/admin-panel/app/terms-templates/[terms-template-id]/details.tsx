"use client"
import React, { useState } from "react"

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
import { formatDate, formatInterval, formatPeriod } from "@/lib/utils"
import { Separator } from "@/components/primitive/separator"
import { Button } from "@/components/primitive/button"
import { UpdateTermsTemplateDialog } from "@/components/terms-template/update-dialog"

gql`
  query TermsTemplate($id: UUID!) {
    termsTemplate(id: $id) {
      id
      name
      termsId
      createdAt
      subjectCanUpdateTermsTemplate
      values {
        duration {
          units
          period
        }
        accrualInterval
        incurrenceInterval
        annualRate
        initialCvl
        marginCallCvl
        liquidationCvl
      }
    }
  }
`

function TermsTemplateDetails({ id }: { id: string }) {
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)
  const { loading, error, data, refetch } = useTermsTemplateQuery({
    variables: { id },
  })

  return (
    <>
      {data?.termsTemplate && (
        <UpdateTermsTemplateDialog
          termsTemplate={data?.termsTemplate}
          openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
          setOpenUpdateTermsTemplateDialog={setOpenUpdateTermsTemplateDialog}
          refetch={refetch}
        />
      )}

      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : data && data.termsTemplate ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div className="flex flex-col gap-1">
                <CardTitle className="flex flex-col space-y-1.5">
                  {data.termsTemplate.name}
                </CardTitle>
                <CardDescription className="text-sm text-textColor-secondary">
                  {data.termsTemplate.termsId}
                </CardDescription>
              </div>
              <Button onClick={() => setOpenUpdateTermsTemplateDialog(true)}>
                Update
              </Button>
            </CardHeader>
            <Separator className="mb-4" />

            <CardContent>
              <DetailsGroup>
                <DetailItem
                  label="Created At"
                  value={formatDate(data.termsTemplate.createdAt)}
                />
                <DetailItem
                  label="Duration"
                  value={`${data.termsTemplate.values.duration.units} ${formatPeriod(data.termsTemplate.values.duration.period)}`}
                />
                <DetailItem
                  label="Accrual Interval"
                  value={formatInterval(data.termsTemplate.values.accrualInterval)}
                />
                <DetailItem
                  label="Incurrence Interval"
                  value={formatInterval(data.termsTemplate.values.incurrenceInterval)}
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
    </>
  )
}

export default TermsTemplateDetails
