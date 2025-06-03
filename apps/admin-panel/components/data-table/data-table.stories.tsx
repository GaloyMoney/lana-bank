import { StoryObj } from "@storybook/nextjs"

import DataTable from "./"

import faker from "@/.storybook/faker"

type User = {
  id: number
  name: string
  email: string
  role: string
  status: "active" | "inactive"
  lastLogin: string
}

const meta = {
  title: "Components/DataTable",
  component: DataTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
}

export default meta
type Story = StoryObj<typeof DataTable<User>>

const generateUsers = (count: number): User[] =>
  Array.from({ length: count }, (_, i) => ({
    id: i + 1,
    name: faker.name.fullName(),
    email: faker.internet.email(),
    role: faker.helpers.arrayElement(["Admin", "User", "Editor"]),
    status: faker.helpers.arrayElement(["active", "inactive"]),
    lastLogin: faker.date.past().toISOString().split("T")[0],
  }))

export const Basic: Story = {
  args: {
    columns: [
      { key: "name", header: "Name" },
      { key: "email", header: "Email" },
      { key: "role", header: "Role" },
    ],
    data: generateUsers(10),
  },
}

export const Loading: Story = {
  args: {
    columns: [
      { key: "name", header: "Name" },
      { key: "email", header: "Email" },
      { key: "role", header: "Role" },
    ],
    loading: true,
  },
}

export const Empty: Story = {
  args: {
    data: [],
  },
}

export const Interactive: Story = {
  args: {
    columns: [
      { key: "name", header: "Name" },
      { key: "email", header: "Email" },
      { key: "role", header: "Role" },
    ],
    data: generateUsers(10),
    onRowClick: (item: User) => alert(`Clicked row with ID: ${item.id}`),
  },
}

export const FullFeatured: Story = {
  args: {
    data: generateUsers(10),
    columns: [
      { key: "name", header: "Name", width: "200px" },
      { key: "email", header: "Email", width: "250px" },
      {
        key: "status",
        header: "Status",
        render: (status: User["status"]) => (
          <span
            className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium capitalize
          ${status === "active" ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"}`}
          >
            {status}
          </span>
        ),
      },
      {
        key: "lastLogin",
        header: "Last Login",
        align: "right",
        render: (date: string) => new Date(date).toLocaleDateString(),
      },
    ],
    onRowClick: (item: User) => alert(`Clicked row with ID: ${item.id}`),
  },
}
