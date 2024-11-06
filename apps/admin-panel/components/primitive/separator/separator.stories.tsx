import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Separator } from "@/components/primitive/separator"

export default {
  title: "Components/Separator",
  component: Separator,
  argTypes: {
    orientation: {
      control: "radio",
      options: ["horizontal", "vertical"],
      description: "Orientation of the separator",
    },
    className: {
      control: "text",
      description: "Additional CSS classes to apply to the separator for styling",
    },
  },
} as Meta<typeof Separator>

const Template: StoryFn<typeof Separator> = (args) => (
  <div
    style={{
      width: "100%",
      height: "50px",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
    }}
  >
    <Separator {...args} />
  </div>
)

export const Horizontal = Template.bind({})
Horizontal.args = {
  orientation: "horizontal",
}

export const Vertical = Template.bind({})
Vertical.args = {
  orientation: "vertical",
  style: { height: "100px" },
}
