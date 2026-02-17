import { useState } from "react"
import type { Meta, StoryObj } from "@storybook/nextjs"

import PaginatedTable, { PaginatedData } from "./"

import faker from "@/.storybook/faker"

type Person = {
  id: string
  name: string
  email: string
  status: "active" | "pending" | "inactive"
  role: "admin" | "user" | "manager"
  joinDate: string
}

const generatePeople = (count: number): Person[] =>
  Array.from({ length: count }, () => ({
    id: faker.string.uuid(),
    name: faker.person.fullName(),
    email: faker.internet.email(),
    status: faker.helpers.arrayElement(["active", "pending", "inactive"]),
    role: faker.helpers.arrayElement(["admin", "user", "manager"]),
    joinDate: faker.date.past().toISOString(),
  }))

const SAMPLE_DATA = generatePeople(75)

const meta: Meta<typeof PaginatedTable<Person>> = {
  title: "Components/PaginatedTable",
  component: PaginatedTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div className="w-[900px] p-4">
        <Story />
      </div>
    ),
  ],
}

export default meta
type Story = StoryObj<typeof PaginatedTable<Person>>

const createPaginatedData = (data: Person[]): PaginatedData<Person> => ({
  edges: data.map((node) => ({ node, cursor: node.id })),
  pageInfo: {
    hasNextPage: false,
    hasPreviousPage: false,
    startCursor: data[0]?.id || "",
    endCursor: data[data.length - 1]?.id || "",
  },
})

export const Basic: Story = {
  args: {
    columns: [
      { key: "name", label: "Name" },
      { key: "email", label: "Email" },
      { key: "role", label: "Role" },
    ],
    data: createPaginatedData(SAMPLE_DATA),
    pageSize: 10,
    loading: false,
  },
}

export const CustomRendering: Story = {
  name: "Custom Remdering (No Header)",
  args: {
    columns: [
      { key: "name", label: "Name" },
      {
        key: "status",
        label: "Status",
        render: (status: Person["status"]) => (
          <span
            className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium capitalize
            ${
              status === "active"
                ? "bg-green-100 text-green-800"
                : status === "pending"
                  ? "bg-yellow-100 text-yellow-800"
                  : "bg-red-100 text-red-800"
            }`}
          >
            {status}
          </span>
        ),
      },
      {
        key: "joinDate",
        label: "Join Date",
        render: (date: string) => new Date(date).toLocaleDateString(),
      },
    ],
    data: createPaginatedData(SAMPLE_DATA),
    pageSize: 10,
    loading: false,
    showHeader: false,
  },
}

export const WithSortingAndFiltering: Story = {
  render: (args) => {
    const [data, setData] = useState(() => createPaginatedData(SAMPLE_DATA))

    const handleSort = (column: keyof Person, direction: "ASC" | "DESC") => {
      const sortedData = [...data.edges].sort((a, b) => {
        const valueA = a.node[column]
        const valueB = b.node[column]
        const compareResult = String(valueA).localeCompare(String(valueB))
        return direction === "ASC" ? compareResult : -compareResult
      })

      setData({
        edges: sortedData,
        pageInfo: data.pageInfo,
      })
    }

    const handleFilter = (filters: Partial<Record<keyof Person, string>>) => {
      const filteredData = SAMPLE_DATA.filter((item) =>
        Object.entries(filters).every(
          ([key, value]) => value === undefined || item[key as keyof Person] === value,
        ),
      )

      setData(createPaginatedData(filteredData))
    }

    return (
      <PaginatedTable<Person>
        {...args}
        data={data}
        onSort={handleSort}
        onFilter={handleFilter}
        columns={[
          {
            key: "name",
            label: "Name",
            sortable: true,
          },
          {
            key: "status",
            label: "Status",
            filterValues: ["active", "pending", "inactive"],
            render: (status: Person["status"]) => (
              <span
                className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium capitalize
                ${
                  status === "active"
                    ? "bg-green-100 text-green-800"
                    : status === "pending"
                      ? "bg-yellow-100 text-yellow-800"
                      : "bg-red-100 text-red-800"
                }`}
              >
                {status}
              </span>
            ),
          },
          {
            key: "role",
            label: "Role",
            filterValues: ["admin", "user", "manager"],
          },
        ]}
      />
    )
  },
  args: {
    pageSize: 10,
    loading: false,
  },
}

export const Loading: Story = {
  args: {
    columns: [
      { key: "name", label: "Name" },
      { key: "email", label: "Email" },
      { key: "role", label: "Role" },
    ],
    data: undefined,
    loading: true,
    pageSize: 10,
  },
}

export const Empty: Story = {
  args: {
    data: createPaginatedData([]),
  },
}
