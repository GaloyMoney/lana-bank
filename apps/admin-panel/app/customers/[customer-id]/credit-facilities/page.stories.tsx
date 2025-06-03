import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import CustomerLayout from "../layout"

import CustomerCreditFacilitiesPage from "./page"

import {
  GetCustomerCreditFacilitiesDocument,
  GetCustomerBasicDetailsDocument,
  AccountStatus,
} from "@/lib/graphql/generated"

const meta = {
  title: "Pages/Customers/Customer/CreditFacilities",
  component: CustomerCreditFacilitiesPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof CustomerCreditFacilitiesPage>

export default meta
type Story = StoryObj<typeof meta>

const mockParams = { "customer-id": "4178b451-c9cb-4841-b248-5cc20e7774a6" }

const layoutMocks = [
  {
    request: {
      query: GetCustomerBasicDetailsDocument,
      variables: {
        id: "4178b451-c9cb-4841-b248-5cc20e7774a6",
      },
    },
    result: {
      data: {
        customer: {
          id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
          customerId: "4178b451-c9cb-4841-b248-5cc20e7774a6",
          email: "test@lana.com",
          telegramId: "test",
          status: AccountStatus.Inactive,
          level: "NOT_KYCED",
          createdAt: "2024-11-25T06:23:56.549713Z",
        },
      },
    },
  },
]

const pageMocks = [
  {
    request: {
      query: GetCustomerCreditFacilitiesDocument,
      variables: {
        id: "4178b451-c9cb-4841-b248-5cc20e7774a6",
      },
    },
    result: {
      data: {
        customer: {
          id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
          creditFacilities: [
            {
              id: "CreditFacility:313789f5-72f7-40bf-8eba-ae5c6b57b588",
              creditFacilityId: "313789f5-72f7-40bf-8eba-ae5c6b57b588",
              collateralizationState: "NO_COLLATERAL",
              status: "PENDING_COLLATERALIZATION",
              createdAt: "2024-11-25T06:25:30.866119Z",
              balance: {
                collateral: {
                  btcBalance: 0,
                },
                outstanding: {
                  usdBalance: 0,
                },
              },
            },
          ],
        },
      },
    },
  },
]

export const Default: Story = {
  args: {
    params: Promise.resolve(mockParams),
  },
  decorators: [
    (Story) => (
      <MockedProvider mocks={layoutMocks} addTypename={false}>
        <CustomerLayout params={Promise.resolve(mockParams)}>
          <MockedProvider mocks={pageMocks} addTypename={false}>
            <Story />
          </MockedProvider>
        </CustomerLayout>
      </MockedProvider>
    ),
  ],
}
