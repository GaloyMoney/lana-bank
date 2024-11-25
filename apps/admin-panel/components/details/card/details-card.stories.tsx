import type { Meta, StoryObj } from "@storybook/react"

import { DetailItemProps } from "../items"

import { DetailsCard } from "./"

import { Button } from "@/ui/button"

const meta: Meta<typeof DetailsCard> = {
  title: "Components/DetailsCard",
  component: DetailsCard,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
}

export default meta
type Story = StoryObj<typeof DetailsCard>

const sampleDetails: DetailItemProps[] = [
  {
    label: "Email",
    value: "john@test.com",
  },
  {
    label: "Phone",
    value: "+1234567890",
  },
  {
    label: "Name",
    value: "John Doe",
  },
  {
    label: "location",
    value: "US",
  },
]

export const Basic: Story = {
  args: {
    title: "Customer Information",
    description: "Basic customer details",
    details: sampleDetails,
  },
}

export const TwoColumns: Story = {
  args: {
    title: "Customer Information",
    description: "Two-column layout",
    details: sampleDetails,
    columns: 2,
  },
}

export const WithError: Story = {
  args: {
    title: "Customer Information",
    details: sampleDetails,
    errorMessage: "Some error occurred",
  },
}

export const WithFooter: Story = {
  args: {
    title: "Customer Information",
    details: sampleDetails,
    footerContent: <Button>Save</Button>,
  },
}
