"use client"
import QRCode from "react-qr-code"
import { useState } from "react"

import { toast } from "sonner"

import { UiNode } from "@ory/client"

import { useRouter } from "next/navigation"

import { IoAdd, IoTrashOutline } from "react-icons/io5"

import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"

import {
  createTotpSetupFlow,
  submitTotpSetupFlow,
} from "@/lib/kratos/public/setup-totp-flow"

export interface AuthenticatorDialogProps {
  open: boolean
  onClose: () => void
  totpSecret: string
  onSubmit: () => void
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  totpCode: string
  error: string | null
}

export const AuthenticatorDialog: React.FC<AuthenticatorDialogProps> = ({
  open,
  onClose,
  totpSecret,
  onSubmit,
  onChange,
  totpCode,
}) => (
  <Dialog
    open={open}
    onOpenChange={() => {
      onClose()
    }}
  >
    <DialogContent className="max-w-96">
      <DialogHeader className="flex flex-col space-y-1.5 text-center">
        <DialogTitle className="text-center">Setup Authenticator</DialogTitle>
        <DialogDescription className="text-center">
          Scan the QR code with your authenticator app and enter the code below
        </DialogDescription>
      </DialogHeader>
      <div className="flex flex-col justify-center items-center gap-4">
        <div className="flex justify-center items-center bg-white p-4 rounded-lg">
          <QRCode size={200} value={totpSecret || ""} />
        </div>
        <div className="bg-secondary p-1 rounded-md px-2 flex gap-2 items-center">
          <p className="text-xs">{totpSecret}</p>
        </div>
        <Input
          value={totpCode}
          onChange={onChange}
          placeholder="Enter the code from your authenticator app"
        />
        <Button className="w-full" onClick={onSubmit}>
          Submit
        </Button>
      </div>
    </DialogContent>
  </Dialog>
)

const SetupAuthenticator = ({ totpUnlinkNode }: { totpUnlinkNode: UiNode | null }) => {
  const router = useRouter()
  const [totpCode, setTotpCode] = useState<string>("")
  const [error, setError] = useState<string | null>(null)
  const [openTotpDialog, setOpenTotpDialog] = useState<boolean>(false)
  const [flowData, setFlowData] = useState<{
    flowId: string
    totpSecret: string
    csrfToken: string
  } | null>(null)

  const handleTotpSetup = async () => {
    const response = await createTotpSetupFlow()

    if (response instanceof Error) {
      setError(response.message)
      return
    }

    const { flowId, totpSecret, csrfToken } = response
    setFlowData({
      flowId,
      totpSecret,
      csrfToken,
    })
    setOpenTotpDialog(true)
  }

  const handleSubmitTotp = async () => {
    if (!flowData || !totpCode) {
      return
    }

    const response = await submitTotpSetupFlow({
      flowId: flowData.flowId,
      totpCode,
      csrfToken: flowData.csrfToken,
    })

    if (response instanceof Error) {
      setError(response.message)
      return
    }

    if (response.success) {
      toast.success("Authenticator app setup successfully")
      setOpenTotpDialog(false)
      router.refresh()
    }
  }

  return (
    <>
      <div className="flex justify-between items-center align-middle">
        <p className="font-semibold leading-none tracking-tight">
          Setup Authenticator App
        </p>
        {totpUnlinkNode === null ? (
          <Button onClick={handleTotpSetup}>
            <IoAdd className="w-5 h-5" />
            <p>Add New</p>
          </Button>
        ) : (
          <Button
            onClick={() => {
              toast.info("This feature is not available yet")
            }}
          >
            <IoTrashOutline className="w-5 h-5" />
            <p className="ml-1">Remove</p>
          </Button>
        )}
      </div>

      {flowData && (
        <AuthenticatorDialog
          open={openTotpDialog}
          onClose={() => {
            setTotpCode("")
            setOpenTotpDialog(!openTotpDialog)
          }}
          totpSecret={flowData.totpSecret}
          onSubmit={handleSubmitTotp}
          onChange={(e) => setTotpCode(e.target.value)}
          totpCode={totpCode}
          error={error}
        />
      )}
    </>
  )
}

export { SetupAuthenticator }
