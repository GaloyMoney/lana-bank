"use client"

import { useEffect, useState } from "react"
import { useSearchParams } from "next/navigation"

import PaginatedTable, { Column, PaginatedData } from "@/components/paginated-table"
import CreateCustomer from "./create"

interface Item {
  id: string
  name: string
  age: number
}

const itemsData: PaginatedData<Item> = {
  edges: [
    { node: { id: "1", name: "Alice", age: 25 } },
    { node: { id: "2", name: "Bob", age: 30 } },
  ],
  pageInfo: {
    endCursor: "cursor2",
    startCursor: "cursor1",
    hasNextPage: true,
    hasPreviousPage: false,
  },
}

const fetchMoreItems = async (cursor: string): Promise<void> => {
  // Fetch more data using the cursor
}

const columns: Column<Item>[] = [
  { key: "id", label: "ID" },
  { key: "name", label: "Name", sortable: true },
  {
    key: "age",
    label: "Age",
    sortable: true,
    filterValues: [25, 30, 35],
    render: (value) => <span>{value} years old</span>,
  },
]

const Customers = () => {
  const searchParams = useSearchParams()

  const [openCreate, setOpenCreate] = useState(false)

  useEffect(() => {
    if (searchParams.get("create")) setOpenCreate(true)
  }, [searchParams, setOpenCreate])

  return (
    <>
      <div className="bg-page rounded-md p-[10px] flex flex-col gap-1 border">
        <div className="text-title-md">Customers</div>
        <div className="!text-body text-body-sm">
          Individuals or entities who hold accounts, loans, or credit facilities with the
          bank
        </div>
        <PaginatedTable<Item>
          columns={columns}
          data={itemsData}
          fetchMore={fetchMoreItems}
        />
      </div>
      <CreateCustomer open={openCreate} setOpen={setOpenCreate} />
    </>
  )
}

export default Customers
