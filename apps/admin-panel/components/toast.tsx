"use client"

import { Toaster as SonnerToast } from "sonner"

type ToasterProps = React.ComponentProps<typeof SonnerToast>

const Toast: React.FC<ToasterProps> = (props) => (
  <SonnerToast
    toastOptions={{
      classNames: {
        toast: "bg-primary text-on-action shadow-lg border-primary",
        description: "text-body",
        actionButton: "bg-primary text-on-action",
        cancelButton: "bg-action-secondary text-on-disabled",
      },
    }}
    {...props}
  />
)

export { Toast }
