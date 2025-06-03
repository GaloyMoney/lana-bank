import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import Deposits from "./page"

import faker from "@/.storybook/faker"

import { DepositsDocument } from "@/lib/graphql/generated"
import { mockDeposit, mockPageInfo } from "@/lib/graphql/generated/mocks"

const createRandomDeposits = () => {
  const count = faker.number.int({ min: 5, max: 10 })
  return Array.from({ length: count }, () => ({
    node: mockDeposit(),
  }))
}

const baseMocks = [
  {
    request: {
      query: DepositsDocument,
      variables: {
        first: 10,
      },
    },
    result: {
      data: {
        deposits: {
          edges: createRandomDeposits(),
          pageInfo: mockPageInfo(),
        },
      },
    },
  },
]

const meta = {
  title: "Pages/Deposits",
  component: Deposits,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof Deposits>

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
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/deposits",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: DepositsDocument,
        variables: {
          first: 10,
        },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <Deposits />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/deposits",
      },
    },
  },
}
