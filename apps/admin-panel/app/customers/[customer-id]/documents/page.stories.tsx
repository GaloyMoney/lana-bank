import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "../layout"

import CustomerDocumentsPage from "./page"

import {
  Activity,
  CustomerType,
  DepositAccountStatus,
  GetCustomerBasicDetailsDocument,
  GetCustomerDocumentsDocument,
  KycVerification,
} from "@/lib/graphql/generated"

const CUSTOMER_ID = "4178b451-c9cb-4841-b248-5cc20e7774a6"

const buildParams = () => Promise.resolve({ "customer-id": CUSTOMER_ID })

const customerDetailsMock: MockedResponse = {
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

const documentsMock: MockedResponse = {
  request: {
    query: GetCustomerDocumentsDocument,
    variables: { id: CUSTOMER_ID },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:1",
        customerId: CUSTOMER_ID,
        documents: [
          {
            __typename: "CustomerDocument",
            id: "doc-001",
            documentId: "doc-001",
            filename: "passport.pdf",
          },
          {
            __typename: "CustomerDocument",
            id: "doc-002",
            documentId: "doc-002",
            filename: "address-proof.pdf",
          },
        ],
      },
    },
  },
}

const emptyDocumentsMock: MockedResponse = {
  request: {
    query: GetCustomerDocumentsDocument,
    variables: { id: CUSTOMER_ID },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:1",
        customerId: CUSTOMER_ID,
        documents: [],
      },
    },
  },
}

type StoryProps = React.ComponentProps<typeof CustomerDocumentsPage> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Documents",
  component: CustomerDocumentsPage,
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

const renderWithLayout = ({ params }: StoryProps) => (
  <CustomerLayout params={params}>
    <CustomerDocumentsPage params={params} />
  </CustomerLayout>
)

export const Default: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, documentsMock],
  },
  render: renderWithLayout,
}

export const Empty: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, emptyDocumentsMock],
  },
  render: renderWithLayout,
}
