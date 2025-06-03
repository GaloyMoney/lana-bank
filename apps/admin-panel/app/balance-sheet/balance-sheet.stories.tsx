import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import BalanceSheet from "./page"

import { BalanceSheetDocument } from "@/lib/graphql/generated"

import { balanceSheetMockData } from "@/.storybook/mocks"

const createMocks = () => [
  {
    request: {
      query: BalanceSheetDocument,
    },
    variableMatcher: () => true,
    result: balanceSheetMockData,
  },
]

const BalanceSheetStory = () => {
  const mocks = createMocks()

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <BalanceSheet />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: BalanceSheetDocument,
      },
      variableMatcher: () => true,
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <BalanceSheet />
    </MockedProvider>
  )
}

const meta = {
  title: "Pages/BalanceSheet",
  component: BalanceSheetStory,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof BalanceSheet>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/balance-sheet",
      },
    },
  },
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/balance-sheet",
      },
    },
  },
}
