import type { Meta, StoryObj } from "@storybook/react"

import LedgerAccountIndexPage from "./page"

const LedgerAccountIndexPageStory = () => {
  return <LedgerAccountIndexPage />
}

const meta: Meta = {
  title: "Pages/LedgerAccounts",
  component: LedgerAccountIndexPageStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
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

