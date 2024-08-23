"use client"
import React, { useState } from "react"
import { useRouter } from "next/navigation"

import LoansTable from "./loans-table"

import { Button } from "@/components/primitive/button"
import { Input } from "@/components/primitive/input"
import { PageHeading } from "@/components/page-heading"

function LoanPage() {
  const router = useRouter()
  const [inputLoanId, setInputLoanId] = useState("")

  const handleSearch = () => {
    router.push(`/loans/${inputLoanId}`)
  }

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Loans</PageHeading>
        <div className="flex gap-2">
          <Input
            onChange={(e) => setInputLoanId(e.target.value)}
            placeholder="Find a loan by loan ID"
            id="loanId"
            name="loanId"
            value={inputLoanId}
            className="w-80"
          />
          <Button onClick={handleSearch} variant="primary">
            Search
          </Button>
        </div>
      </div>

      <LoansTable />
    </main>
  )
}

export default LoanPage
