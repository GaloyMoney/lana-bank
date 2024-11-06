import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Button } from "@/components/primitive/button"

export default {
  title: "Components/Button",
  component: Button,
  argTypes: {
    children: {
      control: "text",
      defaultValue: "Click me!",
    },
    variant: {
      control: { type: "select" },
      options: ["primary", "secondary", "ghost", "transparent", "link", "outline"],
    },
    enabled: {
      control: { type: "boolean" },
      defaultValue: true,
    },
  },
} as Meta<typeof Button>

const Template: StoryFn<typeof Button> = (args) => <Button {...args} />

export const Primary = Template.bind({})
Primary.args = {
  variant: "primary",
  children: "Primary",
}

export const Secondary = Template.bind({})
Secondary.args = {
  variant: "secondary",
  children: "Secondary",
}

export const Ghost = Template.bind({})
Ghost.args = {
  variant: "ghost",
  children: "Ghost",
}

export const Transparent = Template.bind({})
Transparent.args = {
  variant: "transparent",
  children: "Transparent",
}

export const Link = Template.bind({})
Link.args = {
  variant: "link",
  children: "Link",
}

export const Outline = Template.bind({})
Outline.args = {
  variant: "outline",
  children: "Outline",
}

export const Disabled = Template.bind({})
Disabled.args = {
  variant: "primary",
  children: "Disabled",
  enabled: false,
}
