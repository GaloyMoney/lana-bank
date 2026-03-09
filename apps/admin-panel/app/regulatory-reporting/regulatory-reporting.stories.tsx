import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import RegulatoryReportingPage from "./page"

import {
  AvailableReportDefinitionsDocument,
  ReportRunsDocument,
} from "@/lib/graphql/generated"
import {
  mockReportDefinition,
  mockReportRunConnection,
} from "@/lib/graphql/generated/mocks"

const reportRunsMock = {
  request: { query: ReportRunsDocument, variables: { first: 10 } },
  result: { data: { reportRuns: mockReportRunConnection() } },
}

const availableReportDefinitionsMock = {
  request: { query: AvailableReportDefinitionsDocument },
  result: {
    data: {
      availableReportDefinitions: [
        mockReportDefinition({
          reportDefinitionId: "nrp_51/01_saldo_cuenta",
          norm: "nrp_51",
          id: "01_saldo_cuenta",
          friendlyName: "saldo_cuenta",
          supportsAsOf: true,
        }),
        mockReportDefinition({
          reportDefinitionId: "other/calculo_de_riesgo_neto",
          norm: "other",
          id: "calculo_de_riesgo_neto",
          friendlyName: "calculo_de_riesgo_neto",
          supportsAsOf: false,
        }),
      ],
    },
  },
}

const baseMocks = [
  availableReportDefinitionsMock,
  // polling consumes one mock every request
  ...Array.from({ length: 100 }, () => reportRunsMock),
]

const meta = {
  title: "Pages/RegulatoryReporting",
  component: RegulatoryReportingPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof RegulatoryReportingPage>

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
        pathname: "/regulatory-reporting",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: AvailableReportDefinitionsDocument,
      },
      delay: Infinity,
    },
    {
      request: {
        query: ReportRunsDocument,
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <RegulatoryReportingPage />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/regulatory-reporting",
      },
    },
  },
}
