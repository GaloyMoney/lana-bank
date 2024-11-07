"use client"
import React, { useState } from "react"
import { gql } from "@apollo/client"

import Link from "next/link"

import { useRouter } from "next/navigation"

import { TermsTemplate, useTermsTemplatesQuery } from "@/lib/graphql/generated"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { formatPeriod } from "@/lib/utils"
import { UpdateTermsTemplateDialog } from "@/components/terms-template/update-dialog"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"

gql`
  query TermsTemplates {
    termsTemplates {
      id
      name
      termsId
      createdAt
      subjectCanUpdateTermsTemplate
      values {
        annualRate
        accrualInterval
        incurrenceInterval
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
  const router = useRouter()

  const { data, refetch, loading, error } = useTermsTemplatesQuery()
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState<TermsTemplate | null>(null)

  return (
    <main>
      {openUpdateTermsTemplateDialog && (
        <UpdateTermsTemplateDialog
          termsTemplate={openUpdateTermsTemplateDialog}
          openUpdateTermsTemplateDialog={Boolean(openUpdateTermsTemplateDialog)}
          setOpenUpdateTermsTemplateDialog={() => setOpenUpdateTermsTemplateDialog(null)}
          refetch={refetch}
        />
      )}
      <Card>
        <CardHeader>
          <CardTitle>Terms Templates</CardTitle>
          <CardDescription>
            Terms template that can be used with loan and credit facility
          </CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <TableLoadingSkeleton />
          ) : error ? (
            <p className="text-destructive mt-6">{error.message}</p>
          ) : data?.termsTemplates && data.termsTemplates.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Duration</TableHead>
                  <TableHead>Annual Rate</TableHead>
                  <TableHead>Initial CVL</TableHead>
                  <TableHead>MarginCall CVL</TableHead>
                  <TableHead>Liquidation CVL</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data?.termsTemplates.map((termsTemplate) => (
                  <TableRow
                    key={termsTemplate.termsId}
                    className="cursor-pointer"
                    onClick={() =>
                      router.push(`/terms-templates/${termsTemplate.termsId}`)
                    }
                  >
                    <TableCell className="hover:underline">
                      <Link href={`/terms-templates/${termsTemplate.termsId}`}>
                        {termsTemplate.name}
                      </Link>
                    </TableCell>
                    <TableCell>
                      {String(termsTemplate.values.duration.units) +
                        " " +
                        formatPeriod(termsTemplate.values.duration.period)}
                    </TableCell>
                    <TableCell>{termsTemplate.values.annualRate}%</TableCell>
                    <TableCell>{termsTemplate.values.initialCvl}%</TableCell>
                    <TableCell>{termsTemplate.values.marginCallCvl}%</TableCell>
                    <TableCell>{termsTemplate.values.liquidationCvl}%</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm">No data to display</p>
          )}
        </CardContent>
      </Card>
    </main>
  )
}

export default TermPage
