import { useState } from "react"

import type { Meta, StoryObj } from "@storybook/nextjs"

import { DetailsGroup, DetailItem } from "."

const meta: Meta<typeof DetailsGroup> = {
  title: "Components/Details/Group",
  component: DetailsGroup,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
}

export default meta
type Story = StoryObj<typeof DetailsGroup>

const basicDetails = [
  { label: "Address", value: "123 Main St, City, Country" },
  { label: "Email", value: "john.doe@example.com" },
  { label: "Phone", value: "+1 (555) 123-4567" },
  { label: "Full Name", value: "John Doe" },
]

export const Vertical: Story = {
  args: {
    layout: "vertical",
    children: basicDetails.map((detail) => (
      <DetailItem
        key={detail.label.toString()}
        label={detail.label}
        value={detail.value}
      />
    )),
  },
}

export const Horizontal: Story = {
  args: {
    layout: "horizontal",
    children: basicDetails
      .slice(0, 2)
      .map((detail) => (
        <DetailItem
          key={detail.label.toString()}
          label={detail.label}
          value={detail.value}
          className="w-96"
        />
      )),
  },
}

export const TwoColumns: Story = {
  args: {
    columns: 2,
    children: basicDetails.map((detail) => (
      <DetailItem
        key={detail.label.toString()}
        label={detail.label}
        value={detail.value}
      />
    )),
  },
}

export const WithLinks = {
  render: () => (
    <DetailsGroup>
      <DetailItem
        label="Website"
        value="Example.com"
        href="https://example.com"
        showHoverEffect
      />
      <DetailItem
        label="Portfolio"
        value="View Portfolio"
        href="https://portfolio.example.com"
        showHoverEffect
      />
    </DetailsGroup>
  ),
}

export const WithInteractions = {
  render: () => {
    const [clickedValue, setClickedValue] = useState<string | null>(null)

    return (
      <div>
        <DetailsGroup>
          <DetailItem
            label="Clickable Item 1"
            value="Click me!"
            onClick={() => setClickedValue("Item 1 clicked!")}
            showHoverEffect
          />
          <DetailItem
            label="Clickable Item 2"
            value="Click me too!"
            onClick={() => setClickedValue("Item 2 clicked!")}
            showHoverEffect
          />
        </DetailsGroup>
        {clickedValue && (
          <div className="mt-4 p-2 bg-secondary text-secondary-foreground rounded">
            {clickedValue}
          </div>
        )}
      </div>
    )
  },
}
