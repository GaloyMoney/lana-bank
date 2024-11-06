import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Checkbox } from "."

export default {
  title: "Components/Checkbox",
  component: Checkbox,
  argTypes: {
    checked: {
      control: "boolean",
      description: "Controls the checked state of the checkbox",
    },
    disabled: {
      control: "boolean",
      description: "Controls whether the checkbox is disabled",
    },
    onCheckedChange: {
      action: "checkedChange",
      description: "Event handler for when the checked state changes",
    },
  },
} as Meta<typeof Checkbox>

const Template: StoryFn<typeof Checkbox> = (args) => <Checkbox {...args} />

export const Unchecked = Template.bind({})
Unchecked.args = {
  checked: false,
}

export const Checked = Template.bind({})
Checked.args = {
  checked: true,
}

export const Disabled = Template.bind({})
Disabled.args = {
  disabled: true,
}

export const Interactive = Template.bind({})
Interactive.args = {
  checked: false,
  onCheckedChange: (checked: boolean) => console.log("Checkbox is now:", checked),
}
