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
import { useCustomerCreateMutation } from "@/lib/graphql/generated"
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"

gql`
  mutation CustomerCreate($input: CustomerCreateInput!) {
    customerCreate(input: $input) {
      customer {
        customerId
        email
        status
        level
        applicantId
      }
    }
  }
`

type CreateCustomerDialogProps = {
  setOpenCreateCustomerDialog: (isOpen: boolean) => void
  openCreateCustomerDialog: boolean
  refetch?: () => void
}

export const CreateCustomerDialog: React.FC<CreateCustomerDialogProps> = ({
  setOpenCreateCustomerDialog,
  openCreateCustomerDialog,
  refetch,
}) => {
  const router = useRouter()

  const [createCustomer, { loading, reset }] = useCustomerCreateMutation()
  const [email, setEmail] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const { data } = await createCustomer({
        variables: {
          input: {
            email,
          },
        },
      })
      toast.success("customer created successfully")
      if (refetch) refetch()
      setOpenCreateCustomerDialog(false)
      router.push(`/customers/${data?.customerCreate.customer.customerId}`)
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
      open={openCreateCustomerDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateCustomerDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add new customer</DialogTitle>
          <DialogDescription>
            Add a new Customer by providing their email address
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
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button loading={loading}>Submit</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
