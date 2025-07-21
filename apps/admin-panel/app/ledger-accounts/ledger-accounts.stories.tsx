import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import LedgerAccountSearch from "./page"

const LedgerAccountSearchStory = () => (
  <MockedProvider addTypename={false}>
    <LedgerAccountSearch />
  </MockedProvider>
)

const meta = {
  title: "Pages/LedgerAccountSearch",
  component: LedgerAccountSearchStory,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof LedgerAccountSearch>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/ledger-accounts",
      },
    },
  },
}
