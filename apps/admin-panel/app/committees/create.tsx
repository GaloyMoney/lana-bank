"use client"

import React, { useState } from "react"
import { toast } from "sonner"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"

import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import { useCreateCommitteeMutation } from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation CreateCommittee($input: CommitteeCreateInput!) {
    committeeCreate(input: $input) {
      committee {
        ...CommitteeFields
      }
    }
  }
`

type CreateCommitteeDialogProps = {
  setOpenCreateCommitteeDialog: (isOpen: boolean) => void
  openCreateCommitteeDialog: boolean
}

export const CreateCommitteeDialog: React.FC<CreateCommitteeDialogProps> = ({
  setOpenCreateCommitteeDialog,
  openCreateCommitteeDialog,
}) => {
  const t = useTranslations("Committees.CommitteeDetails.create")
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => {
      setOpenCreateCommitteeDialog(false)
      resetForm()
    },
  })

  const [createCommittee, { loading, reset, error: createCommitteeError }] =
    useCreateCommitteeMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            committees: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const isLoading = loading || isNavigating

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
      await createCommittee({
        variables: {
          input: {
            name: formValues.name,
          },
        },
        onCompleted: (data) => {
          if (data?.committeeCreate.committee) {
            toast.success(t("success"))
            navigate(`/committees/${data.committeeCreate.committee.committeeId}`)
          } else {
            throw new Error(t("errors.failed"))
          }
        },
      })
    } catch (error) {
      console.error("Error creating committee:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createCommitteeError?.message) {
        setError(createCommitteeError.message)
      } else {
        setError(t("errors.general"))
      }
      toast.error(t("errors.failed"))
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
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="name">{t("fields.name")}</Label>
            <Input
              id="name"
              name="name"
              type="text"
              required
              placeholder={t("placeholders.name")}
              value={formValues.name}
              onChange={handleChange}
              disabled={isLoading}
              data-testid="committee-create-name-input"
            />
          </div>

          {error && <p className="text-destructive">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              data-testid="committee-create-submit-button"
            >
              {t("buttons.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
