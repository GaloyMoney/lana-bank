"use client"

import { Toaster as Toast } from "sonner"

type ToasterProps = React.ComponentProps<typeof Toast>

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Toast
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            "group toast group-[.toaster]:bg-primary-foreground group-[.toaster]:text-textColor-primary group-[.toaster]:shadow-lg group-[.toaster]:border-secondary",
          description: "group-[.toast]:text-textColor-secondary",
          actionButton:
            "group-[.toast]:bg-primary group-[.toast]:text-primary-foreground",
          cancelButton:
            "group-[.toast]:bg-secondary group-[.toast]:text-secondary-foreground",
        },
      }}
      {...props}
    />
  )
}

export { Toaster }
