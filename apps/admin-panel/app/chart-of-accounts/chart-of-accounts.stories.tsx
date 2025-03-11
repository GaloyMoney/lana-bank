import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import ChartOfAccounts from "./page"

import { ChartOfAccountsDocument } from "@/lib/graphql/generated"

import { regularChartOfAccountsMockData } from "@/.storybook/mocks"

const createMocks = () => [
  {
    request: {
      query: ChartOfAccountsDocument,
    },
    result: regularChartOfAccountsMockData,
  },
]

const ChartOfAccountsStory = () => {
  const mocks = createMocks()

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ChartOfAccounts />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: ChartOfAccountsDocument,
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ChartOfAccounts />
    </MockedProvider>
  )
}

const meta = {
  title: "Pages/ChartOfAccounts",
  component: ChartOfAccountsStory,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof ChartOfAccounts>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/chart-of-accounts",
      },
    },
  },
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/chart-of-accounts",
      },
    },
  },
}
