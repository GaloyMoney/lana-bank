"use client"

import React, { useState } from "react"
import { gql, useApolloClient } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"
import { toast } from "sonner"

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
  GetCustomerBasicDetailsDocument,
  useCreateDepositAsyncMutation,
} from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation CreateDepositAsync($input: DepositRecordAsyncInput!) {
    depositRecordAsync(input: $input) {
      deposit {
        ...DepositFields
        account {
          customer {
            id
            customerId
            depositAccount {
              id
              deposits {
                ...DepositFields
              }
            }
            depositAccount {
              id
              balance {
                settled
                pending
              }
            }
          }
        }
      }
    }
  }
`

type CreateDepositDialogProps = {
  setOpenCreateDepositDialog: (isOpen: boolean) => void
  openCreateDepositDialog: boolean
  depositAccountId: string
}

export const CreateDepositDialog: React.FC<CreateDepositDialogProps> = ({
  setOpenCreateDepositDialog,
  openCreateDepositDialog,
  depositAccountId,
}) => {
  const t = useTranslations("Deposits.CreateDepositDialog")
  const [createDepositAsync, { loading, reset }] = useCreateDepositAsyncMutation({
    update: (cache) => {
      cache.modify({
        fields: {
          deposits: (_, { DELETE }) => DELETE,
        },
      })
      cache.gc()
    },
  })
  const [amount, setAmount] = useState<string>("")
  const [reference, setReference] = useState<string>("")
  const [error, setError] = useState<string | null>(null)
  const client = useApolloClient()
  const router = useRouter()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const result = await createDepositAsync({
        variables: {
          input: {
            depositAccountId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
            reference,
          },
        },
      })
      if (result.data) {
        await client.query({
          query: GetCustomerBasicDetailsDocument,
          variables: {
            id: result.data.depositRecordAsync.deposit.account.customer.customerId,
          },
          fetchPolicy: "network-only",
        })
        toast.success(t("success"))
        handleCloseDialog()
        router.push(`/deposits/${result.data.depositRecordAsync.deposit.publicId}`)
      } else {
        throw new Error(t("errors.noData"))
      }
    } catch (error) {
      console.error("Error creating deposit:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const resetStates = () => {
    setAmount("")
    setReference("")
    setError(null)
    reset()
  }

  const handleCloseDialog = () => {
    setOpenCreateDepositDialog(false)
    resetStates()
  }

  return (
    <Dialog open={openCreateDepositDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="amount">{t("fields.amount")}</Label>
            <div className="flex items-center gap-1">
              <Input
                data-testid="deposit-amount-input"
                id="amount"
                type="number"
                required
                placeholder={t("placeholders.amount")}
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                endAdornment={`USD`}
              />
            </div>
          </div>
          <div>
            <Label htmlFor="reference">{t("fields.reference")}</Label>
            <Input
              id="reference"
              type="text"
              placeholder={t("placeholders.reference")}
              value={reference}
              onChange={(e) => setReference(e.target.value)}
            />
          </div>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button type="submit" disabled={loading} data-testid="deposit-submit-button">
              {loading ? t("buttons.submitting") : t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
