import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import CustomerLayout from "./layout"

import { AccountStatus, GetCustomerBasicDetailsDocument } from "@/lib/graphql/generated"

const meta = {
  title: "Pages/Customers/Customer/Layout",
  component: CustomerLayout,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof CustomerLayout>

export default meta
type Story = StoryObj<typeof meta>

const mockParams = { "customer-id": "4178b451-c9cb-4841-b248-5cc20e7774a6" }

const baseMocks = [
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
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <CustomerLayout params={Promise.resolve(mockParams)}>
        <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
      </CustomerLayout>
    </MockedProvider>
  )
}

export const Default: Story = {
  args: {
    params: Promise.resolve(mockParams),
    children: (
      <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
    ),
  },
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={false}>
        <Story />
      </MockedProvider>
    ),
  ],
}

export const Loading: Story = {
  args: {
    params: Promise.resolve(mockParams),
    children: (
      <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
    ),
  },
  render: LoadingStory,
}
