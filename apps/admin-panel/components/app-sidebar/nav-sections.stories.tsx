import type { Meta, StoryObj } from "@storybook/react"

import { Home, Users, LayoutGrid, ClipboardList } from "lucide-react"

import { NavSection } from "./nav-section"

import { SidebarProvider } from "@/ui/sidebar"

const demoNavItems = [
  { title: "Dashboard", url: "/dashboard", icon: Home },
  { title: "Customers", url: "/customers", icon: Users },
  { title: "Products", url: "/products", icon: LayoutGrid },
  { title: "Orders", url: "/orders", icon: ClipboardList },
]

const meta = {
  title: "Components/AppSidebar/NavSection",
  component: NavSection,
  parameters: {
    layout: "centered",
  },
  decorators: [
    (Story) => (
      <div className="w-64 border rounded p-4">
        <SidebarProvider>
          <Story />
        </SidebarProvider>
      </div>
    ),
  ],
} satisfies Meta<typeof NavSection>

export default meta
type Story = StoryObj<typeof NavSection>

export const Default: Story = {
  args: {
    items: demoNavItems,
    label: "Main Navigation",
  },
}
