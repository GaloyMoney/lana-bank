import type { Meta, StoryObj } from "@storybook/react"
import { ApolloError } from "@apollo/client"
import { MockedProvider } from "@apollo/client/testing"

import Dashboard from "./page"

import faker from "@/.storybook/faker"

import {
  DashboardDocument,
  GetRealtimePriceUpdatesDocument,
  AllActionsDocument,
  ApprovalProcessStatus,
} from "@/lib/graphql/generated"

import {
  mockDashboard,
  mockApprovalProcess,
  mockRealtimePrice,
  mockPageInfo,
} from "@/lib/graphql/generated/mocks"
import { Satoshis, UsdCents } from "@/types"

interface DashboardStoryArgs {
  activeFacilities: number
  pendingFacilities: number
  totalDisbursedUSD: number
  totalCollateralBTC: number
  btcPriceUSD: number
  numberOfActions: number
  showEmptyActions: boolean
}

const DEFAULT_ARGS: DashboardStoryArgs = {
  activeFacilities: faker.number.int({ min: 3, max: 20 }),
  pendingFacilities: faker.number.int({ min: 0, max: 10 }),
  totalDisbursedUSD: faker.number.int({ min: 1000, max: 100000 }),
  totalCollateralBTC: faker.number.float({ min: 0.01, max: 5, fractionDigits: 5 }),
  btcPriceUSD: faker.number.int({ min: 30000, max: 60000 }),
  numberOfActions: faker.number.int({ min: 3, max: 8 }),
  showEmptyActions: false,
}

const createActions = (args: DashboardStoryArgs) => {
  if (args.showEmptyActions) return []
  return Array.from({ length: args.numberOfActions }, () => {
    return {
      node: mockApprovalProcess({
        status: ApprovalProcessStatus.InProgress,
        subjectCanSubmitDecision: true,
      }),
    }
  })
}

const createMocks = (args: DashboardStoryArgs) => {
  const actions = createActions(args)

  return [
    {
      request: { query: DashboardDocument },
      result: {
        data: {
          dashboard: mockDashboard({
            activeFacilities: args.activeFacilities,
            pendingFacilities: args.pendingFacilities,
            totalDisbursed: args.totalDisbursedUSD as UsdCents,
            totalCollateral: args.totalCollateralBTC as Satoshis,
          }),
        },
      },
    },
    {
      request: { query: GetRealtimePriceUpdatesDocument },
      result: {
        data: {
          realtimePrice: mockRealtimePrice({
            usdCentsPerBtc: args.btcPriceUSD as UsdCents,
          }),
        },
      },
    },
    {
      request: { query: AllActionsDocument },
      result: {
        data: {
          approvalProcesses: {
            pageInfo: mockPageInfo(),
            edges: actions,
          },
        },
      },
    },
  ]
}

const DashboardStory = (args: DashboardStoryArgs) => {
  const mocks = createMocks(args)

  return (
    <div className="max-w-7xl m-auto p-4">
      <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
        <Dashboard />
      </MockedProvider>
    </div>
  )
}

const meta: Meta<typeof DashboardStory> = {
  title: "Pages/Dashboard",
  component: DashboardStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  argTypes: {
    activeFacilities: {
      control: { type: "number", min: 0, max: 10000 },
      description: "Number of active facilities",
    },
    pendingFacilities: {
      control: { type: "number", min: 0, max: 10000 },
      description: "Number of pending facilities",
    },
    totalDisbursedUSD: {
      control: { type: "number", min: 0, max: 10000000, step: 0.01 },
      description: "Total amount disbursed (in USD)",
    },
    totalCollateralBTC: {
      control: { type: "number", min: 0, max: 100, step: 0.00000001 },
      description: "Total collateral (in BTC)",
    },
    btcPriceUSD: {
      control: { type: "number", min: 1, max: 1000000, step: 0.01 },
      description: "Bitcoin price (in USD)",
    },
    numberOfActions: {
      control: { type: "number", min: 0, max: 20 },
      description: "Number of pending actions to display",
    },
    showEmptyActions: {
      control: "boolean",
      description: "Show empty state for actions list",
    },
  },
}

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = { args: DEFAULT_ARGS }

export const Error: Story = {
  args: DEFAULT_ARGS,
  render: () => {
    const errorMocks = [
      {
        request: { query: DashboardDocument },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
      {
        request: { query: GetRealtimePriceUpdatesDocument },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
      {
        request: { query: AllActionsDocument },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <Dashboard />
      </MockedProvider>
    )
  },
}
