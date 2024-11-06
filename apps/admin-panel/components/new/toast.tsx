"use client"

import { Toaster as SonnerToast } from "sonner"

type ToasterProps = React.ComponentProps<typeof SonnerToast>

const Toast: React.FC<ToasterProps> = (props) => (
  <SonnerToast
    toastOptions={{
      classNames: {
        toast: "bg-action-secondary text-on-action shadow-lg",
        description: "text-body",
        actionButton: "bg-page text-body",
        cancelButton: "bg-action-secondary text-on-disabled",
      },
    }}
    {...props}
  />
)

export { Toast }
