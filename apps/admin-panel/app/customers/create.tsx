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

  const [createCustomer, { loading, reset, error: createCustomerError }] =
    useCustomerCreateMutation()
  const [email, setEmail] = useState<string>("")
  const [telegramId, setTelegramId] = useState<string>("")
  const [error, setError] = useState<string | null>(null)
  const [isConfirmationStep, setIsConfirmationStep] = useState<boolean>(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!isConfirmationStep) {
      setIsConfirmationStep(true)
      return
    }

    try {
      const { data } = await createCustomer({
        variables: {
          input: {
            email,
            telegramId,
          },
        },
      })
      if (data?.customerCreate.customer) {
        toast.success("Customer created successfully")
        if (refetch) refetch()
        setOpenCreateCustomerDialog(false)
        router.push(`/customers/${data.customerCreate.customer.customerId}`)
      } else {
        throw new Error("Failed to create customer. Please try again.")
      }
    } catch (error) {
      console.error("Error creating customer:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createCustomerError?.message) {
        setError(createCustomerError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to create customer")
    }
  }

  const resetStates = () => {
    setEmail("")
    setTelegramId("")
    setError(null)
    setIsConfirmationStep(false)
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
          <DialogTitle>
            {isConfirmationStep ? "Confirm Customer Details" : "Add new customer"}
          </DialogTitle>
          <DialogDescription>
            {isConfirmationStep
              ? "Please review the details before submitting"
              : "Add a new Customer by providing their email address and Telegram ID"}
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          {isConfirmationStep ? (
            <>
              <div>
                <Label>Email</Label>
                <p>{email}</p>
              </div>
              <div>
                <Label>Telegram ID</Label>
                <p>{telegramId}</p>
              </div>
            </>
          ) : (
            <>
              <div>
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  required
                  placeholder="Please enter the email address"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </div>
              <div>
                <Label htmlFor="telegramId">Telegram ID</Label>
                <Input
                  id="telegramId"
                  type="text"
                  required
                  placeholder="Please enter the Telegram ID"
                  value={telegramId}
                  onChange={(e) => setTelegramId(e.target.value)}
                />
              </div>
            </>
          )}
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            {isConfirmationStep && (
              <Button
                type="button"
                className="text-primary"
                variant="ghost"
                onClick={() => setIsConfirmationStep(false)}
              >
                Back
              </Button>
            )}
            <Button type="submit" loading={loading}>
              {isConfirmationStep ? "Confirm and Submit" : "Review Details"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
