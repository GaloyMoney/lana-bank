import React, { useState } from "react"
import { toast } from "sonner"

import { gql } from "@apollo/client"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/ui/dialog"
import { CustomersDocument, useCustomerCreateMutation } from "@/lib/graphql/generated"
import { Input } from "@/ui/input"
import { Button } from "@/ui/button"
import { Label } from "@/ui/label"

import { useModalNavigation } from "@/hooks/use-modal-navigation"

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
}

export const CreateCustomerDialog: React.FC<CreateCustomerDialogProps> = ({
  setOpenCreateCustomerDialog,
  openCreateCustomerDialog,
}) => {
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => setOpenCreateCustomerDialog(false),
  })

  const [createCustomer, { loading, reset, error: createCustomerError }] =
    useCustomerCreateMutation({
      refetchQueries: [CustomersDocument],
    })

  const isLoading = loading || isNavigating
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
      await createCustomer({
        variables: {
          input: {
            email,
            telegramId,
          },
        },
        onCompleted: (data) => {
          if (data?.customerCreate.customer) {
            toast.success("Customer created successfully")
            navigate(`/customers/${data.customerCreate.customer.customerId}`)
          } else {
            throw new Error("Failed to create customer. Please try again.")
          }
        },
      })
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
    } finally {
      resetStates()
    }
  }

  const resetStates = () => {
    setEmail("")
    setTelegramId("")
    setError(null)
    setIsConfirmationStep(false)
  }

  return (
    <Dialog
      open={openCreateCustomerDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateCustomerDialog(isOpen)
        if (!isOpen) {
          resetStates()
          reset()
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
                  data-testid="customer-create-email"
                  placeholder="Please enter the email address"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  disabled={isLoading}
                />
              </div>
              <div>
                <Label htmlFor="telegramId">Telegram ID</Label>
                <Input
                  id="telegramId"
                  type="text"
                  required
                  data-testid="customer-create-telegram-id"
                  placeholder="Please enter the Telegram ID"
                  value={telegramId}
                  onChange={(e) => setTelegramId(e.target.value)}
                  disabled={isLoading}
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
                disabled={isLoading}
              >
                Back
              </Button>
            )}
            <Button
              type="submit"
              loading={isLoading}
              data-testid="customer-create-submit-button"
            >
              {isConfirmationStep ? "Confirm and Submit" : "Review Details"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
