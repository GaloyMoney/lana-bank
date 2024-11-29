import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import CreateButton, { CreateContextProvider } from "../create"

import Dashboard from "./page"

import { AppSidebar } from "@/components/app-sidebar"

import { SidebarInset, SidebarProvider } from "@/ui/sidebar"

import faker from "@/.storybook/faker"

import {
  DashboardDocument,
  GetRealtimePriceUpdatesDocument,
  AllActionsDocument,
  ApprovalProcessStatus,
  Role,
  AvatarDocument,
} from "@/lib/graphql/generated"

import {
  mockDashboard,
  mockApprovalProcess,
  mockRealtimePrice,
  mockPageInfo,
} from "@/lib/graphql/generated/mocks"
import { Satoshis, UsdCents } from "@/types"
import { RealtimePriceUpdates } from "@/components/realtime-price"

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
    {
      request: { query: AvatarDocument },
      result: {
        data: {
          me: {
            user: {
              userId: "usr_123",
              email: "demo@example.com",
              roles: [Role.Admin],
            },
          },
        },
      },
    },
  ]
}

const DashboardWithSidebar = (args: DashboardStoryArgs) => {
  const mocks = createMocks(args)

  return (
    <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
      <div className={` antialiased select-none bg-background`}>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset className="min-h-screen md:peer-data-[variant=inset]:shadow-none border">
            <CreateContextProvider>
              <div className="container mx-auto p-2">
                <div className="max-w-7xl w-full mx-auto">
                  <header className="flex justify-between items-center mb-2">
                    <div className="font-semibold text-sm p-2 bg-secondary rounded-md">
                      Welcome to Lana Bank
                    </div>
                    <CreateButton />
                  </header>

                  <RealtimePriceUpdates />
                  <main>
                    <Dashboard />
                  </main>
                </div>
              </div>
            </CreateContextProvider>
          </SidebarInset>
        </SidebarProvider>
      </div>
    </MockedProvider>
  )
}

const meta: Meta<typeof DashboardWithSidebar> = {
  title: "Pages/Dashboard",
  component: DashboardWithSidebar,
  parameters: {
    layout: "fullscreen",
    nextjs: { appDirectory: true },
    backgrounds: {
      default: "light",
    },
  },
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
