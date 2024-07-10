import React from "react"

import { Account } from "./accounts"

import { PageHeading } from "@/components/page-heading"
import { chartOfAccountsQuery } from "@/lib/graphql/query/get-chart-of-accounts"
import { Table, TableBody, TableCell, TableRow } from "@/components/primitive/table"

async function ChartOfAccountsPage() {
  const data = await chartOfAccountsQuery()
  if (data instanceof Error) return data

  return (
    <main>
      <PageHeading>{data.chartOfAccounts?.name}</PageHeading>
      <Table>
        <TableBody>
          {data.chartOfAccounts?.categories.map((category) => (
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
