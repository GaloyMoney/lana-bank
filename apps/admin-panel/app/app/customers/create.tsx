"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { toast } from "sonner"
import { gql } from "@apollo/client"
import Dialog from "@material-tailwind/react/components/Dialog"

import { useCreateCustomerMutation } from "@/lib/graphql/generated"
import { Input, Button } from "@/components"

gql`
  mutation CreateCustomer($input: CustomerCreateInput!) {
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

type CreateCustomerProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
}

const CreateCustomer: React.FC<CreateCustomerProps> = ({ setOpen, open }) => {
  const [email, setEmail] = useState<string>("")
  const [telegramId, setTelegramId] = useState<string>("")

  const [createCustomer, { loading, reset, error: createCustomerError }] =
    useCreateCustomerMutation()

  const router = useRouter()

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
        setOpen(false)
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
      open={open}
      handler={() => {
        setOpen(false)
        resetStates()
      }}
    >
      <div className="text-title-md">
        {isConfirmationStep ? "Confirm Customer Details" : "Add new customer"}
      </div>

      <div className="!text-body text-body-sm">
        {isConfirmationStep
          ? "Please review the details before submitting"
          : "Add a new Customer by providing their email address and Telegram ID"}
      </div>
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
      </form>
    </Dialog>
  )
}

export default CreateCustomer
