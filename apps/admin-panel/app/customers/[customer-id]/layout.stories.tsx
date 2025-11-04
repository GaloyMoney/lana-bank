import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "./layout"

import {
  Activity,
  CustomerType,
  DepositAccountStatus,
  GetCustomerBasicDetailsDocument,
  KycVerification,
} from "@/lib/graphql/generated"

const CUSTOMER_ID = "4178b451-c9cb-4841-b248-5cc20e7774a6"

const buildParams = () => Promise.resolve({ "customer-id": CUSTOMER_ID })

const baseMock = {
  request: {
    query: GetCustomerBasicDetailsDocument,
    variables: { id: CUSTOMER_ID },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:1",
        customerId: CUSTOMER_ID,
        publicId: "CUS-001",
        email: "test@lana.com",
        telegramId: "telegramUser",
        kycVerification: KycVerification.Verified,
        activity: Activity.Active,
        level: "LEVEL_2",
        customerType: CustomerType.Individual,
        createdAt: "2024-11-25T06:23:56.549713Z",
        depositAccount: {
          __typename: "DepositAccount",
          id: "DepositAccount:1",
          status: DepositAccountStatus.Active,
          publicId: "DEP-001",
          depositAccountId: "dep-account-123",
          balance: {
            __typename: "DepositAccountBalance",
            settled: 1500000,
            pending: 250000,
          },
          ledgerAccounts: {
            __typename: "DepositAccountLedgerAccounts",
            depositAccountId: "ledger-acc-123",
            frozenDepositAccountId: "ledger-acc-frozen-123",
          },
        },
      },
    },
  },
}

const loadingMock = {
  request: {
    query: GetCustomerBasicDetailsDocument,
    variables: { id: CUSTOMER_ID },
  },
  delay: Infinity,
}

type StoryProps = React.ComponentProps<typeof CustomerLayout> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Layout",
  component: CustomerLayout,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
  argTypes: {
    mocks: { control: false },
  },
}

export default meta

type Story = StoryObj<StoryProps>

const tabPlaceholder = (
  <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
)

export const Default: Story = {
  args: {
    params: buildParams(),
    children: tabPlaceholder,
    mocks: [baseMock],
  },
  render: ({ params, children }) => (
    <CustomerLayout params={params}>{children}</CustomerLayout>
  ),
}

export const Loading: Story = {
  args: {
    params: buildParams(),
    children: tabPlaceholder,
    mocks: [loadingMock],
  },
  render: ({ params, children }) => (
    <CustomerLayout params={params}>{children}</CustomerLayout>
  ),
}
