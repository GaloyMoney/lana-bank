import React, { useState } from "react"
import { StoryFn, Meta } from "@storybook/nextjs"

import { AuthenticatorDialog, AuthenticatorDialogProps } from "./index"

export default {
  title: "Components/AuthenticatorDialog",
  component: AuthenticatorDialog,
} as Meta

const Template: StoryFn<AuthenticatorDialogProps> = (args) => {
  const [open, setOpen] = useState(false)

  return (
    <>
      <button onClick={() => setOpen(true)}>Open Dialog</button>
      <AuthenticatorDialog {...args} open={open} onClose={() => setOpen(false)} />
    </>
  )
}

export const Default = Template.bind({})
Default.args = {
  totpSecret: "1234567890ABCD",
  totpCode: "",
  error: null,
  onSubmit: () => alert("Submit clicked!"),
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => {
    console.log(e.target.value)
  },
}
