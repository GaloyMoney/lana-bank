"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { toast } from "sonner"
import { gql } from "@apollo/client"

import { useCreateCustomerMutation, CustomersDocument } from "@/lib/graphql/generated"
import { Input, Button, Dialog } from "@/components"

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
    useCreateCustomerMutation({
      refetchQueries: [CustomersDocument],
    })

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
        router.push(`/app/customers/${data.customerCreate.customer.customerId}`)
        toast.success("Customer created successfully")
        setOpen(false)
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
    <Dialog open={open} setOpen={setOpen} onClose={resetStates}>
      <div className="">
        <div className="text-title-md">
          {isConfirmationStep ? "Confirm Customer Details" : "Add new customer"}
        </div>

        <div className="!text-body text-body-sm mt-2">
          {isConfirmationStep
            ? "Please review the details before submitting"
            : "Add a new Customer by providing their email address and Telegram ID"}
        </div>
      </div>
      <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
        {isConfirmationStep ? (
          <>
            <div>
              <label className="text-title-sm">Email</label>
              <p className="text-body-md">{email}</p>
            </div>
            <div>
              <label className="text-title-sm">Telegram ID</label>
              <p className="text-body-md">{telegramId}</p>
            </div>
          </>
        ) : (
          <>
            <div>
              <Input
                type="email"
                label="Email"
                required
                placeholder="Please enter the email address"
                onChange={setEmail}
              />
            </div>
            <div>
              <Input
                type="text"
                label="Telegram ID"
                required
                placeholder="Please enter the customer's telegram ID"
                onChange={setTelegramId}
              />
            </div>
          </>
        )}
        {error && <p className="text-error">{error}</p>}
        <div className="flex justify-between">
          {isConfirmationStep && (
            <Button
              type="button"
              variant="outlined"
              onClick={() => setIsConfirmationStep(false)}
              title="Back"
            />
          )}
          <Button
            type="submit"
            variant={isConfirmationStep ? "filled" : "outlined"}
            loading={loading}
            title={isConfirmationStep ? "Confirm and Submit" : "Review Details"}
          />
        </div>
      </form>
    </Dialog>
  )
}

export default CreateCustomer
