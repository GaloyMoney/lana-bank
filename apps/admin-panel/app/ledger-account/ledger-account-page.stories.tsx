import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import LedgerAccountIndexPage from "./page"

const LedgerAccountIndexPageStory = () => {
  return (
    <MockedProvider>
      <LedgerAccountIndexPage />
    </MockedProvider>
  )
}

const meta: Meta = {
  title: "Pages/LedgerAccounts",
  component: LedgerAccountIndexPageStory,
  parameters: {
    layout: "fullscreen",
    nextjs: { appDirectory: true },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/ledger-account`,
      },
    },
  },
}
