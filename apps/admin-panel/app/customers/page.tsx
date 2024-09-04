"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { gql } from "@apollo/client"

import CustomerTable from "./list"
import { CreateCustomerDialog } from "./create"

import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { PageHeading } from "@/components/page-heading"
import { isEmail, isUUID } from "@/lib/utils"

gql`
  query getCustomerByCustomerEmail($email: String!) {
    customerByEmail(email: $email) {
      customerId
      email
      status
      level
      applicantId
      balance {
        checking {
          settled
          pending
        }
      }
    }
  }

  query getCustomerByCustomerId($id: UUID!) {
    customer(id: $id) {
      customerId
      email
      status
      level
      applicantId
      balance {
        checking {
          settled
          pending
        }
      }
    }
  }
`

function CustomerPage({ searchParams }: { searchParams: { search?: string } }) {
  const { search } = searchParams
  const [searchInput, setSearchInput] = useState(search || "")
  const [openCreateCustomerDialog, setOpenCreateCustomerDialog] = useState(false)
  const router = useRouter()

  const handleOpenCreateCustomerDialog = (e: React.FormEvent) => {
    e.preventDefault()
    setOpenCreateCustomerDialog(true)
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    if (searchInput) {
      let searchType = "unknown"
      if (isUUID(searchInput)) {
        searchType = "customerId"
      } else if (isEmail(searchInput)) {
        searchType = "email"
      }
      router.push(
        `/customers?search=${encodeURIComponent(searchInput)}&searchType=${searchType}`,
      )
    } else {
      router.push("/customers")
    }
  }

  const handleClear = () => {
    setSearchInput("")
    router.push("/customers")
  }

  const searchType = search
    ? isUUID(search)
      ? "customerId"
      : isEmail(search)
        ? "email"
        : "unknown"
    : undefined

  return (
    <main>
      <form className="flex justify-between items-center mb-8" onSubmit={handleSearch}>
        <PageHeading className="mb-0">Customers</PageHeading>
        <div className="flex gap-2">
          <Input
            placeholder="Find a customer by ID or email"
            id="search"
            name="search"
            className="w-80"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
          />
          <Button variant="secondary" type="submit">
            Search
          </Button>
          {search && (
            <Button variant="secondary" type="button" onClick={handleClear}>
              X Clear
            </Button>
          )}
          <Button onClick={handleOpenCreateCustomerDialog}>Create New</Button>
        </div>
      </form>
      <CustomerTable
        searchValue={search}
        searchType={searchType as "customerId" | "email" | "unknown" | undefined}
        renderCreateCustomerDialog={(refetch) => (
          <CreateCustomerDialog
            setOpenCreateCustomerDialog={setOpenCreateCustomerDialog}
            openCreateCustomerDialog={openCreateCustomerDialog}
            refetch={refetch}
          />
        )}
      />
    </main>
  )
}

export default CustomerPage
