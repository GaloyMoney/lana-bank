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

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

import {
  User,
  SortDirection,
  UsersSort,
  useCreateCommitteeMutation,
  useUsersQuery,
} from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"
import { camelToScreamingSnake } from "@/lib/utils"

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

  const [sortBy, setSortBy] = useState<UsersSort | null>(null)

  const { data: userData, loading: usersLoading, fetchMore } = useUsersQuery({
    variables: { first: DEFAULT_PAGESIZE, sort: sortBy },
    skip: !openCreateCommitteeDialog,
  })

  const isLoading = loading || isNavigating

  const [name, setName] = useState("")
  const [members, setMembers] = useState<Record<string, string>>({})
  const [error, setError] = useState<string | null>(null)

  const memberUserIds = Object.keys(members)

  const handleToggleMember = (user: User) => {
    setMembers((prev) => {
      const next = { ...prev }
      if (user.userId in next) {
        delete next[user.userId]
      } else {
        next[user.userId] = user.email
      }
      return next
    })
  }

  const handleRemoveMember = (userId: string) => {
    setMembers((prev) => {
      const next = { ...prev }
      delete next[userId]
      return next
    })
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
            name,
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
    setName("")
    setMembers({})
    setError(null)
    reset()
  }

  const columns: Column<User>[] = [
    {
      key: "email",
      label: t("columns.email"),
      sortable: true,
    },
    {
      key: "role",
      label: t("columns.role"),
      render: (role) => role?.name || "",
    },
    {
      key: "userId",
      label: "",
      render: (userId) =>
        userId in members ? (
          <span className="text-xs font-medium text-primary">{t("selected")}</span>
        ) : null,
    },
  ]

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
      <DialogContent className="sm:max-w-2xl">
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
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isLoading}
              data-testid="committee-create-name-input"
            />
          </div>

          <div className="space-y-6">
            <div>
              <Label>{t("fields.selectedMembers")}</Label>
              {memberUserIds.length > 0 ? (
                <div className="mt-2 flex flex-wrap gap-1.5">
                  {memberUserIds.map((userId) => (
                    <span
                      key={userId}
                      className="inline-flex items-center gap-1 rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium"
                    >
                      {members[userId]}
                      <button
                        type="button"
                        onClick={() => handleRemoveMember(userId)}
                        disabled={isLoading}
                        className="ml-0.5 inline-flex items-center rounded-full p-0.5 hover:bg-muted-foreground/20 transition-colors"
                      >
                        ✕
                      </button>
                    </span>
                  ))}
                </div>
              ) : (
                <p className="mt-2 text-sm text-muted-foreground">
                  {t("fields.noMembersSelected")}
                </p>
              )}
            </div>

            <div>
              <Label>{t("fields.users")}</Label>
              <div className="mt-2">
                <PaginatedTable<User>
                  columns={columns}
                  data={userData?.users as PaginatedData<User>}
                  loading={usersLoading}
                  fetchMore={async (cursor) =>
                    fetchMore({ variables: { after: cursor } })
                  }
                  pageSize={DEFAULT_PAGESIZE}
                  style="compact"
                  onClick={handleToggleMember}
                  onSort={(column, direction) => {
                    setSortBy({
                      by: camelToScreamingSnake(column as string) as UsersSort["by"],
                      direction: direction as SortDirection,
                    })
                  }}
                />
              </div>
            </div>
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
