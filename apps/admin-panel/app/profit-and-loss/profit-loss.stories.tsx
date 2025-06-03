import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import ProfitAndLoss from "./page"

import { ProfitAndLossStatementDocument } from "@/lib/graphql/generated"

import { profitLossMockData } from "@/.storybook/mocks"

const createMocks = () => [
  {
    request: {
      query: ProfitAndLossStatementDocument,
    },
    variableMatcher: () => true,
    result: profitLossMockData,
  },
]

const ProfitAndLossStory = () => {
  const mocks = createMocks()

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ProfitAndLoss />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: ProfitAndLossStatementDocument,
      },
      variableMatcher: () => true,
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ProfitAndLoss />
    </MockedProvider>
  )
}

const meta = {
  title: "Pages/ProfitAndLoss",
  component: ProfitAndLossStory,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof ProfitAndLoss>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/profit-and-loss",
      },
    },
  },
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/profit-and-loss",
      },
    },
  },
}
