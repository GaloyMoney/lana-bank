import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "./layout"
import CustomerCreditFacilitiesLandingPage from "./page"

import {
  Activity,
  CollateralizationState,
  CreditFacilityStatus,
  CustomerType,
  DepositAccountStatus,
  GetCustomerBasicDetailsDocument,
  GetCustomerCreditFacilitiesDocument,
  KycVerification,
} from "@/lib/graphql/generated"

const CUSTOMER_ID = "4178b451-c9cb-4841-b248-5cc20e7774a6"

const buildParams = () => Promise.resolve({ "customer-id": CUSTOMER_ID })

const customerDetailsMock = {
  request: {
    query: GetCustomerBasicDetailsDocument,
    variables: {
      id: CUSTOMER_ID,
    },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
        customerId: CUSTOMER_ID,
        email: "test@lana.com",
        telegramId: "telegramUser",
        kycVerification: KycVerification.Verified,
        activity: Activity.Active,
        level: "LEVEL_2",
        customerType: CustomerType.Individual,
        createdAt: "2024-11-25T06:23:56.549713Z",
        publicId: "CUS-001",
        depositAccount: {
          __typename: "DepositAccount",
          id: "DepositAccount:123",
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

const creditFacilitiesMock = {
  request: {
    query: GetCustomerCreditFacilitiesDocument,
    variables: {
      id: CUSTOMER_ID,
    },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
        creditFacilities: [
          {
            __typename: "CreditFacility",
            id: "CreditFacility:cf-001",
            creditFacilityId: "cf-001",
            publicId: "CF-001",
            collateralizationState: CollateralizationState.NoCollateral,
            status: CreditFacilityStatus.Active,
            activatedAt: "2024-02-10T09:00:00.000Z",
            balance: {
              __typename: "CreditFacilityBalance",
              collateral: {
                __typename: "CollateralBalance",
                btcBalance: 150_000_000,
              },
              outstanding: {
                __typename: "Outstanding",
                usdBalance: 5_000_000,
              },
            },
          },
        ],
      },
    },
  },
}

const emptyCreditFacilitiesMock = {
  request: {
    query: GetCustomerCreditFacilitiesDocument,
    variables: {
      id: CUSTOMER_ID,
    },
  },
  result: {
    data: {
      customerByPublicId: {
        __typename: "Customer",
        id: "Customer:4178b451-c9cb-4841-b248-5cc20e7774a6",
        creditFacilities: [],
      },
    },
  },
}

type StoryProps = React.ComponentProps<typeof CustomerCreditFacilitiesLandingPage> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Landing",
  component: CustomerCreditFacilitiesLandingPage,
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

export const Default: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, creditFacilitiesMock],
  },
  render: ({ params }) => (
    <CustomerLayout params={params}>
      <CustomerCreditFacilitiesLandingPage params={params} />
    </CustomerLayout>
  ),
}

export const Empty: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, emptyCreditFacilitiesMock],
  },
  render: ({ params }) => (
    <CustomerLayout params={params}>
      <CustomerCreditFacilitiesLandingPage params={params} />
    </CustomerLayout>
  ),
}
