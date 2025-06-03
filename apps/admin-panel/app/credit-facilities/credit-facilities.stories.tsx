import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import CreditFacilities from "./page"

import faker from "@/.storybook/faker"

import { CreditFacilitiesDocument } from "@/lib/graphql/generated"
import { mockCreditFacility, mockPageInfo } from "@/lib/graphql/generated/mocks"

const createRandomFacilities = () => {
  const count = faker.number.int({ min: 5, max: 10 })
  return Array.from({ length: count }, () => ({
    node: mockCreditFacility(),
  }))
}

const baseMocks = [
  {
    request: {
      query: CreditFacilitiesDocument,
      variables: {
        first: 10,
        sort: null,
        filter: null,
      },
    },
    result: {
      data: {
        creditFacilities: {
          edges: createRandomFacilities(),
          pageInfo: mockPageInfo(),
        },
      },
    },
  },
]

const meta = {
  title: "Pages/CreditFacilities",
  component: CreditFacilities,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
      navigation: {
        pathname: "/credit-facilities",
      },
    },
  },
} satisfies Meta<typeof CreditFacilities>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={false}>
        <Story />
      </MockedProvider>
    ),
  ],
}
