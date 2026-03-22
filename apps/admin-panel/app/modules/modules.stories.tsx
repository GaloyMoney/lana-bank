import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import Modules from "./page"

import { DepositAccountConfigDocument, CreditFacilityConfigDocument } from "@/lib/graphql/generated"

const baseMocks = [
  {
    request: {
      query: DepositAccountConfigDocument,
    },
    result: {
      data: {
        depositAccountConfig: null,
      },
    },
  },
  {
    request: {
      query: CreditFacilityConfigDocument,
    },
    result: {
      data: {
        creditFacilityConfig: null,
      },
    },
  },
]

const meta = {
  title: "Pages/Modules",
  component: Modules,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof Modules>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={false}>
        <Story />
      </MockedProvider>
    ),
  ],
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/modules",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: DepositAccountConfigDocument,
      },
      delay: Infinity,
    },
    {
      request: {
        query: CreditFacilityConfigDocument,
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <Modules />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/modules",
      },
    },
  },
}

const DataStory = () => {
  const mocks = [
    {
      request: {
        query: DepositAccountConfigDocument,
      },
      result: {
        data: {
          depositAccountConfig: {
            chartOfAccountsDepositAccountsParentCode: "41.01.0101",
            chartOfAccountsOmnibusParentCode: "51.01",
          },
        },
      },
    },
    {
      request: {
        query: CreditFacilityConfigDocument,
      },
      result: {
        data: {
          creditFacilityConfig: {
            chartOfAccountFacilityOmnibusParentCode: "41.01.0101",
            chartOfAccountCollateralOmnibusParentCode: "51.01",
            chartOfAccountFacilityParentCode: "41.01.0101",
            chartOfAccountCollateralParentCode: "51.01",
            chartOfAccountDisbursedReceivableParentCode: "41.01.0101",
            chartOfAccountInterestReceivableParentCode: "51.01",
            chartOfAccountInterestIncomeParentCode: "41.01.0101",
            chartOfAccountFeeIncomeParentCode: "51.01",
            chartOfAccountPaymentHoldingParentCode: "11.99.0201",
          },
        },
      },
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <Modules />
    </MockedProvider>
  )
}

export const Data: Story = {
  render: DataStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/modules",
      },
    },
  },
}
