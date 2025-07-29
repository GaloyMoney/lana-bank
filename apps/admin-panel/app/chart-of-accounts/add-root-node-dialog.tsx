"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { useModalNavigation } from "@/hooks/use-modal-navigation"

import {
  useChartOfAccountsAddRootNodeMutation,
  useChartOfAccountsForLedgerQuery,
  DebitOrCredit,
} from "@/lib/graphql/generated"

gql`
  mutation ChartOfAccountsAddRootNode($input: ChartOfAccountsAddRootNodeInput!) {
    chartOfAccountsAddRootNode(input: $input) {
      chartOfAccounts {
        ...ChartOfAccountsFields
      }
    }
  }
`

type AddRootNodeDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export const AddRootNodeDialog: React.FC<AddRootNodeDialogProps> = ({
  open,
  onOpenChange,
}) => {
  const t = useTranslations("ChartOfAccounts.AddRootNodeDialog")
  const [addRootNode, { loading }] = useChartOfAccountsAddRootNodeMutation()
  const { data: chartData } = useChartOfAccountsForLedgerQuery()

  const [code, setCode] = useState("")
  const [name, setName] = useState("")
  const [normalBalanceType, setNormalBalanceType] = useState<DebitOrCredit | "">("")
  const [error, setError] = useState<string | null>(null)

  const { navigate } = useModalNavigation({
    closeModal: () => {
      resetForm()
      onOpenChange(false)
    },
  })

  const validateAccountCode = (value: string): string => {
    const cleaned = value.replace(/[^0-9]/g, "")
    return cleaned.slice(0, 14)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const trimmedCode = code.trim()
    const trimmedName = name.trim()

    if (!trimmedCode || !trimmedName || !normalBalanceType) {
      setError(t("errors.required"))
      return
    }

    if (!chartData?.chartOfAccounts?.chartId) {
      setError(t("errors.chartNotFound"))
      return
    }

    try {
      await addRootNode({
        variables: {
          input: {
            chartId: chartData.chartOfAccounts.chartId,
            code: trimmedCode,
            name: trimmedName,
            normalBalanceType: normalBalanceType as DebitOrCredit,
          },
        },
      })

      toast.success(t("success"))
      resetForm()
      navigate(`/ledger-accounts/${trimmedCode}`)
    } catch (error) {
      console.error("Error adding root node:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const resetForm = () => {
    setCode("")
    setName("")
    setNormalBalanceType("")
    setError(null)
  }

  const handleClose = () => {
    resetForm()
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="code">
              {t("fields.code")} <span className="text-destructive">*</span>
            </Label>
            <Input
              data-testid="root-node-code-input"
              id="code"
              type="text"
              required
              autoFocus
              placeholder={t("placeholders.code")}
              value={code}
              onChange={(e) => setCode(validateAccountCode(e.target.value))}
              maxLength={14}
            />
          </div>

          <div>
            <Label htmlFor="name">
              {t("fields.name")} <span className="text-destructive">*</span>
            </Label>
            <Input
              data-testid="root-node-name-input"
              id="name"
              type="text"
              required
              placeholder={t("placeholders.name")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <div>
            <Label htmlFor="normalBalanceType">
              {t("fields.normalBalanceType")} <span className="text-destructive">*</span>
            </Label>
            <Select
              value={normalBalanceType}
              onValueChange={(value) => setNormalBalanceType(value as DebitOrCredit)}
            >
              <SelectTrigger data-testid="root-node-balance-type-select">
                <SelectValue placeholder={t("placeholders.normalBalanceType")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={DebitOrCredit.Debit}>
                  {t("balanceTypes.debit")}
                </SelectItem>
                <SelectItem value={DebitOrCredit.Credit}>
                  {t("balanceTypes.credit")}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
              disabled={loading}
            >
              {t("buttons.cancel")}
            </Button>
            <Button
              type="submit"
              disabled={loading}
              data-testid="root-node-submit-button"
            >
              {loading ? t("buttons.adding") : t("buttons.add")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
