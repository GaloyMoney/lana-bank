import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import CustomerPage from "./page"
import CustomerLayout from "./layout"

import {
  GetCustomerOverviewDocument,
  GetCustomerBasicDetailsDocument,
  AccountStatus,
} from "@/lib/graphql/generated"

const meta = {
  title: "Pages/Customers/Customer/Overview",
  component: CustomerPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof CustomerPage>

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

const overviewMocks = [
  {
    request: {
      query: GetCustomerOverviewDocument,
      variables: {
        id: "4178b451-c9cb-4841-b248-5cc20e7774a6",
      },
    },
    result: {
      data: {
        customer: {
          id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
          customerId: "4178b451-c9cb-4841-b248-5cc20e7774a6",
          balance: {
            checking: {
              settled: 1000,
              pending: 500,
            },
          },
        },
      },
    },
  },
]

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: GetCustomerBasicDetailsDocument,
        variables: {
          id: "4178b451-c9cb-4841-b248-5cc20e7774a6",
        },
      },
      delay: Infinity,
    },
    {
      request: {
        query: GetCustomerOverviewDocument,
        variables: {
          id: "4178b451-c9cb-4841-b248-5cc20e7774a6",
        },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider
      defaultOptions={{ watchQuery: { fetchPolicy: "no-cache" } }}
      mocks={mocks}
      addTypename={false}
    >
      <CustomerLayout params={mockParams}>
        <CustomerPage params={mockParams} />
      </CustomerLayout>
    </MockedProvider>
  )
}

export const Default: Story = {
  args: {
    params: mockParams,
  },
  decorators: [
    (Story) => (
      <MockedProvider
        defaultOptions={{ watchQuery: { fetchPolicy: "no-cache" } }}
        mocks={layoutMocks}
        addTypename={false}
      >
        <CustomerLayout params={mockParams}>
          <MockedProvider
            defaultOptions={{ watchQuery: { fetchPolicy: "no-cache" } }}
            mocks={overviewMocks}
            addTypename={false}
          >
            <Story />
          </MockedProvider>
        </CustomerLayout>
      </MockedProvider>
    ),
  ],
}

export const Loading: Story = {
  args: {
    params: mockParams,
  },
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/customers/[customer-id]",
      },
    },
  },
}
