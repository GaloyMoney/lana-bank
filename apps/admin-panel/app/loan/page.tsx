import React from "react"
import { redirect } from "next/navigation"

import { Button } from "@/components/primitive/button"
import { Input } from "@/components/primitive/input"
import { Label } from "@/components/primitive/label"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { PageHeading } from "@/components/page-heading"
import { getLoanDetails } from "@/lib/graphql/query/get-loan"
import { DetailItem, DetailsGroup } from "@/components/details"

const searchLoan = async (formData: FormData) => {
  "use server"
  const loanId = formData.get("loanId")
  if (!loanId || typeof loanId !== "string") {
    redirect(`/loan`)
  }
  redirect(`/loan?loanId=${loanId}`)
}

async function LoanPage({
  searchParams,
}: {
  searchParams: {
    loanId: string
  }
}) {
  const loanId = searchParams.loanId
  let loanDetails = null

  if (loanId) {
    loanDetails = await getLoanDetails({ id: loanId })
  }

  return (
    <main>
      <PageHeading>Loan</PageHeading>
      <div className="mt-4 mb-4 max-w-[30rem]">
        <Label htmlFor="loanId">Loan ID</Label>
        <form className="flex gap-2" action={searchLoan}>
          <Input placeholder="Find a loan by loan ID" id="loanId" name="loanId" />
          <Button variant="secondary">Search</Button>
        </form>
      </div>

      <Card className="max-w-[60rem]">
        {loanDetails instanceof Error ? (
          <CardContent className="pt-6">{loanDetails.message}</CardContent>
        ) : loanDetails?.loan ? (
          <>
            <CardHeader>
              <h2 className="font-semibold leading-none tracking-tight">Loan Details</h2>
              <p className="text-textColor-secondary text-sm">
                {loanDetails.loan.loanId}
              </p>
            </CardHeader>
            <CardContent>
              <Separator className="mb-6" />
              <DetailsGroup>
                <DetailItem label="User ID" value={loanDetails.loan.userId} />
                <DetailItem
                  label="BTC collateral balance"
                  value={loanDetails.loan.balance.collateral.btcBalance}
                />
                <DetailItem
                  label="USD Outstanding balance"
                  value={loanDetails.loan.balance.interestIncurred.usdBalance}
                />
                <DetailItem
                  label="Interest Incurred"
                  value={loanDetails.loan.balance.interestIncurred.usdBalance}
                />
              </DetailsGroup>
            </CardContent>
          </>
        ) : null}
      </Card>
    </main>
  )
}

export default LoanPage
