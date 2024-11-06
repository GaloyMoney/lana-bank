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
import { useCreateCommitteeMutation } from "@/lib/graphql/generated"
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"

gql`
  mutation CreateCommittee($input: CommitteeCreateInput!) {
    committeeCreate(input: $input) {
      committee {
        id
        committeeId
        createdAt
        currentMembers {
          userId
          email
          roles
        }
      }
    }
  }
`

type CreateCommitteeDialogProps = {
  setOpenCreateCommitteeDialog: (isOpen: boolean) => void
  openCreateCommitteeDialog: boolean
  refetch?: () => void
}

export const CreateCommitteeDialog: React.FC<CreateCommitteeDialogProps> = ({
  setOpenCreateCommitteeDialog,
  openCreateCommitteeDialog,
  refetch,
}) => {
  const router = useRouter()

  const [createCommittee, { loading, reset, error: createCommitteeError }] =
    useCreateCommitteeMutation()

  const [formValues, setFormValues] = useState({
    name: "",
  })

  const [error, setError] = useState<string | null>(null)

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target
    setFormValues((prevValues) => ({
      ...prevValues,
      [name]: value,
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const { data } = await createCommittee({
        variables: {
          input: {
            name: formValues.name,
          },
        },
      })

      if (data?.committeeCreate.committee) {
        toast.success("Committee created successfully")
        if (refetch) refetch()
        setOpenCreateCommitteeDialog(false)
        router.push(`/committees/${data.committeeCreate.committee.committeeId}`)
      } else {
        throw new Error("Failed to create committee. Please try again.")
      }
    } catch (error) {
      console.error("Error creating committee:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createCommitteeError?.message) {
        setError(createCommitteeError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to create committee")
    }
  }

  const resetForm = () => {
    setFormValues({
      name: "",
    })
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openCreateCommitteeDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateCommitteeDialog(isOpen)
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Committee</DialogTitle>
          <DialogDescription>
            Create a new committee by providing the required information
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="name">Committee Name</Label>
            <Input
              id="name"
              name="name"
              type="text"
              required
              placeholder="Enter the committee name"
              value={formValues.name}
              onChange={handleChange}
            />
          </div>

          {error && <p className="text-destructive">{error}</p>}

          <DialogFooter>
            <Button type="submit" loading={loading}>
              Create Committee
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
