import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { useUserCreateMutation } from "@/lib/graphql/generated"
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"
import { sendMagicLinkToEmail } from "@/lib/user/server-actions/send-magic-link"

gql`
  mutation UserCreate($input: UserCreateInput!) {
    userCreate(input: $input) {
      user {
        userId
        email
        roles
      }
    }
  }
`

function CreateUserDialog({
  setOpenCreateUserDialog,
  openCreateUserDialog,
  refetch,
}: {
  setOpenCreateUserDialog: (isOpen: boolean) => void
  openCreateUserDialog: boolean
  refetch: () => void
}) {
  const router = useRouter()

  const [createUser, { loading, reset }] = useUserCreateMutation()
  const [email, setEmail] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createUser({
        variables: {
          input: {
            email,
          },
        },
        onCompleted: async (data) => {
          refetch()
          toast.success("User created successfully")
          await sendMagicLinkToEmail(email)
          setOpenCreateUserDialog(false)
          router.push(`/users/${data.userCreate.user.userId}`)
        },
      })
    } catch (error) {
      console.error(error)
      if (error instanceof Error) {
        setError(error.message)
      }
    }
    resetStates()
  }

  const resetStates = () => {
    setEmail("")
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openCreateUserDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateUserDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add new User</DialogTitle>
          <DialogDescription>
            Add a new user to the admin-panel by providing their email address
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label>Email</Label>
            <Input
              type="email"
              required
              placeholder="Please enter the email address"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <p className="text-textColor-secondary text-xs ml-1">
            A magic link will be sent to the email address provided.
          </p>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button loading={loading}>Submit</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CreateUserDialog
