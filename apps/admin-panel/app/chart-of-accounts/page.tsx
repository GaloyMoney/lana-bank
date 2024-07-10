"use client"
import React from "react"

import { Account } from "./accounts"

import { PageHeading } from "@/components/page-heading"
import { Table, TableBody, TableCell, TableRow } from "@/components/primitive/table"
import { useGetChartOfAccountsQuery } from "@/lib/graphql/generated"

function ChartOfAccountsPage() {
  const { data, loading } = useGetChartOfAccountsQuery()
  if (loading) return <p>Loading...</p>

  return (
    <main>
      <PageHeading>{data?.chartOfAccounts?.name}</PageHeading>
      <Table>
        <TableBody>
          {data?.chartOfAccounts?.categories
            // without the sort, the categories are being displayed randomly
            .toSorted(({ name: str1 }, { name: str2 }) =>
              str1 < str2 ? -1 : +(str1 > str2),
            )
            .map((category) => (
              <>
                <TableRow key={category.id}>
                  <TableCell className="text-primary font-bold uppercase tracking-widest leading-8">
                    {category.name}
                  </TableCell>
                </TableRow>
                {category.accounts.map((account) => (
                  <Account key={account.id} account={account} />
                ))}
              </>
            ))}
        </TableBody>
      </Table>
    </main>
  )
}

export default ChartOfAccountsPage
