"use client"

import { Dialog as MTDialog } from "@/lib/ui/mtw"

type DialogProps = {
  open: boolean
  setOpen: (isOpen: boolean) => void
  onClose?: () => void
}

const Dialog: React.FC<React.PropsWithChildren<DialogProps>> = ({
  open,
  setOpen,
  onClose,
  children,
}) => {
  return (
    <MTDialog
      className="p-[20px] flex flex-col gap-4"
      open={open}
      handler={() => {
        setOpen(false)
        onClose && onClose()
      }}
      placeholder={undefined}
      onPointerEnterCapture={undefined}
      onPointerLeaveCapture={undefined}
    >
      {children}
    </MTDialog>
  )
}

export default Dialog
