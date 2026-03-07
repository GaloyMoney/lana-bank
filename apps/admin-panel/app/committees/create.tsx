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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { useCreateCommitteeMutation, useUsersQuery } from "@/lib/graphql/generated"
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

  const { data: userData, loading: usersLoading } = useUsersQuery()

  const isLoading = loading || isNavigating

  const [formValues, setFormValues] = useState({
    name: "",
  })
  const [selectedUserId, setSelectedUserId] = useState<string>("")
  const [memberUserIds, setMemberUserIds] = useState<string[]>([])

  const [error, setError] = useState<string | null>(null)

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target
    setFormValues((prevValues) => ({
      ...prevValues,
      [name]: value,
    }))
  }

  const handleAddMember = () => {
    if (selectedUserId && !memberUserIds.includes(selectedUserId)) {
      setMemberUserIds((prev) => [...prev, selectedUserId])
      setSelectedUserId("")
    }
  }

  const handleRemoveMember = (userId: string) => {
    setMemberUserIds((prev) => prev.filter((id) => id !== userId))
  }

  const getUserEmail = (userId: string) => {
    const user = userData?.users.find((u) => u.userId === userId)
    return user?.email || userId
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (memberUserIds.length === 0) {
      setError(t("errors.atLeastOneMember"))
      return
    }

    try {
      await createCommittee({
        variables: {
          input: {
            name: formValues.name,
            memberUserIds,
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
    setSelectedUserId("")
    setMemberUserIds([])
    setError(null)
    reset()
  }

  const availableUsers = userData?.users.filter(
    (user) => !memberUserIds.includes(user.userId),
  )

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

          <div>
            <Label>{t("fields.members")}</Label>
            <div className="flex gap-2">
              <Select
                value={selectedUserId}
                onValueChange={setSelectedUserId}
                disabled={isLoading || usersLoading}
              >
                <SelectTrigger data-testid="committee-create-member-select">
                  <SelectValue placeholder={t("placeholders.selectMember")} />
                </SelectTrigger>
                <SelectContent>
                  {availableUsers?.map((user) => (
                    <SelectItem key={user.userId} value={user.userId}>
                      {user.email} {user.role?.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                type="button"
                variant="outline"
                onClick={handleAddMember}
                disabled={!selectedUserId || isLoading}
                data-testid="committee-create-add-member-button"
              >
                {t("buttons.addMember")}
              </Button>
            </div>
            {memberUserIds.length > 0 && (
              <ul className="mt-2 space-y-1">
                {memberUserIds.map((userId) => (
                  <li
                    key={userId}
                    className="flex items-center justify-between rounded bg-muted px-2 py-1 text-sm"
                  >
                    <span>{getUserEmail(userId)}</span>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveMember(userId)}
                      disabled={isLoading}
                    >
                      ✕
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          {error && <p className="text-destructive">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              disabled={memberUserIds.length === 0}
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
