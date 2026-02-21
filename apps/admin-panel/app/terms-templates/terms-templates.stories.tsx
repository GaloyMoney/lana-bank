import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import TermPage from "./page"

import { DisbursalPolicy, TermsTemplatesDocument } from "@/lib/graphql/generated"
import {
  mockDuration,
  mockTermsTemplate,
  mockTermValues,
} from "@/lib/graphql/generated/mocks"

const templates = [
  {
    name: "High Risk",
    riskProfile: {
      annualRate: 18,
      liquidationCvl: 85,
      marginCallCvl: 90,
      initialCvl: 95,
      duration: {
        units: 12,
      },
    },
  },
  {
    name: "Medium Risk",
    riskProfile: {
      annualRate: 13,
      liquidationCvl: 87,
      marginCallCvl: 92,
      initialCvl: 96,
      duration: {
        units: 12,
      },
    },
  },
  {
    name: "Preferred Customer",
    riskProfile: {
      annualRate: 10,
      liquidationCvl: 88,
      marginCallCvl: 93,
      initialCvl: 97,
      duration: {
        units: 12,
      },
    },
  },
  {
    name: "Prime Customer",
    riskProfile: {
      annualRate: 6,
      liquidationCvl: 89,
      marginCallCvl: 94,
      initialCvl: 98,
      duration: {
        units: 12,
      },
    },
  },
]

const createTermsTemplates = () => {
  return templates.map((template) => {
    const { riskProfile } = template
    return mockTermsTemplate({
      name: template.name,
      values: mockTermValues({
        annualRate: riskProfile.annualRate,
        liquidationCvl: {
          __typename: "FiniteCvlPct",
          value: riskProfile.liquidationCvl,
        },
        marginCallCvl: {
          __typename: "FiniteCvlPct",
          value: riskProfile.marginCallCvl,
        },
        initialCvl: {
          __typename: "FiniteCvlPct",
          value: riskProfile.initialCvl,
        },
        oneTimeFeeRate: 0.5,
        disbursalPolicy: DisbursalPolicy.MultipleDisbursal,
        duration: mockDuration({
          units: riskProfile.duration.units,
        }),
      }),
    })
  })
}

const baseMocks = [
  {
    request: {
      query: TermsTemplatesDocument,
    },
    result: {
      data: {
        termsTemplates: createTermsTemplates(),
      },
    },
  },
]

const meta = {
  title: "Pages/TermsTemplates",
  component: TermPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof TermPage>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks}>
        <Story />
      </MockedProvider>
    ),
  ],
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/terms-templates",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: TermsTemplatesDocument,
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <TermPage />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/terms-templates",
      },
    },
  },
}
