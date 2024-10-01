"use client"
import React, { useState } from "react"
import { gql } from "@apollo/client"

import Link from "next/link"

import { CreateTermsTemplateDialog } from "./create"

import { PageHeading } from "@/components/page-heading"
import { Button } from "@/components/primitive/button"
import { useTermsTemplatesQuery } from "@/lib/graphql/generated"
import { Card, CardContent } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { formatPeriod } from "@/lib/utils"

gql`
  query TermsTemplates {
    termsTemplates {
      id
      name
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
  const { data, refetch, loading, error } = useTermsTemplatesQuery()
  const [openCreateUserDialog, setOpenCreateUserDialog] = useState<boolean>(false)

  return (
    <main>
      <CreateTermsTemplateDialog
        openCreateTermsTemplateDialog={openCreateUserDialog}
        setOpenCreateTermsTemplateDialog={setOpenCreateUserDialog}
        refetch={refetch}
      />
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Terms Template</PageHeading>
        <Button onClick={() => setOpenCreateUserDialog(true)}>Create New</Button>
      </div>
      <Card>
        <CardContent>
          {loading ? (
            <p className="mt-6">Loading...</p>
          ) : error ? (
            <p className="text-destructive mt-6">{error.message}</p>
          ) : data?.termsTemplates && data.termsTemplates.length > 0 ? (
            <Table className="mt-6">
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
                  <TableRow key={termsTemplate.termsId}>
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
            <p className="mt-6">No data found</p>
          )}
        </CardContent>
      </Card>
    </main>
  )
}

export default TermPage
