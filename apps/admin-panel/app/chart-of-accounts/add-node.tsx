"use client"

import React, { useState, useEffect } from "react"
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
  useChartOfAccountsAddNodeMutation,
  DebitOrCredit,
  useValidateParentAccountCodeLazyQuery,
} from "@/lib/graphql/generated"

gql`
  query ValidateParentAccountCode($code: String!) {
    ledgerAccountByCode(code: $code) {
      id
      code
      name
    }
  }

  mutation ChartOfAccountsAddNode($input: ChartOfAccountsAddNodeInput!) {
    chartOfAccountsAddNode(input: $input) {
      chartOfAccounts {
        ...ChartOfAccountsFields
      }
    }
  }
`

type AddChartNodeDialogProps = {
  setOpenAddNodeDialog: (isOpen: boolean) => void
  openAddNodeDialog: boolean
  chartId: string
  parentCode?: string
}

export const AddChartNodeDialog: React.FC<AddChartNodeDialogProps> = ({
  setOpenAddNodeDialog,
  openAddNodeDialog,
  chartId,
  parentCode,
}) => {
  const t = useTranslations("ChartOfAccounts.AddNodeDialog")
  const [addNode, { loading, reset }] = useChartOfAccountsAddNodeMutation()

  const [code, setCode] = useState<string>("")
  const [name, setName] = useState<string>("")
  const [normalBalanceType, setNormalBalanceType] = useState<DebitOrCredit | "">("")
  const [parent, setParent] = useState<string>(parentCode || "")
  const [error, setError] = useState<string | null>(null)
  const [parentAccountName, setParentAccountName] = useState<string | null>(null)
  const [isValidatingParent, setIsValidatingParent] = useState(false)
  const [validationKey, setValidationKey] = useState(0)

  const handleCloseDialog = () => {
    setOpenAddNodeDialog(false)
    resetStates()
  }

  const { navigate, isNavigating } = useModalNavigation({
    closeModal: handleCloseDialog,
  })

  const [validateParentCode] = useValidateParentAccountCodeLazyQuery({})

  useEffect(() => {
    if (openAddNodeDialog) {
      setParentAccountName(null)
      setIsValidatingParent(false)
      setParent(parentCode || "")
      if (parentCode) {
        setCode(parentCode + ".")
      }
      setValidationKey((prev) => prev + 1)
    }
  }, [openAddNodeDialog, parentCode])

  useEffect(() => {
    if (!parent.trim()) {
      setParentAccountName(null)
      setIsValidatingParent(false)
      return
    }

    setIsValidatingParent(true)
    setParentAccountName(null)

    const timer = setTimeout(async () => {
      const result = await validateParentCode({
        variables: { code: parent.trim() },
        fetchPolicy: "network-only",
      })
      if (result.data?.ledgerAccountByCode) {
        setParentAccountName(result.data.ledgerAccountByCode.name)
        const parentCode = parent.trim()
        const expectedPrefix = parentCode + "."
        if (!code || !code.startsWith(expectedPrefix)) {
          setCode(expectedPrefix)
        }
      }
      setIsValidatingParent(false)
    }, 400)

    return () => clearTimeout(timer)
  }, [parent, validationKey])

  const validateAccountCodeInput = (value: string): string => {
    return value.replace(/[^0-9.]/g, "")
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const result = await addNode({
        variables: {
          input: {
            chartId,
            parent: parent.trim() || undefined,
            code: code.trim(),
            name: name.trim(),
            normalBalanceType: normalBalanceType as DebitOrCredit,
          },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        navigate(`/ledger-account/${code.trim()}`)
      } else {
        throw new Error(t("errors.noData"))
      }
    } catch (error) {
      console.error("Error adding chart node:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const resetStates = () => {
    setCode("")
    setName("")
    setNormalBalanceType("")
    setError(null)
    reset()
  }

  return (
    <Dialog open={openAddNodeDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="parent">{t("fields.parent")}</Label>
            <Input
              data-testid="chart-node-parent-input"
              id="parent"
              type="text"
              placeholder={t("placeholders.parent")}
              value={parent}
              onChange={(e) => setParent(validateAccountCodeInput(e.target.value))}
            />
            {isValidatingParent && (
              <p className="text-sm text-muted-foreground mt-1">{t("validating")}...</p>
            )}
            {parentAccountName && (
              <p className="text-sm mt-1 ml-1"> {parentAccountName}</p>
            )}
            {parent.trim() && !isValidatingParent && !parentAccountName && (
              <p className="text-sm text-destructive mt-1">{t("parentNotFound")}</p>
            )}
          </div>

          <div>
            <Label htmlFor="code">
              {t("fields.code")} <span className="text-destructive">*</span>
            </Label>
            <Input
              data-testid="chart-node-code-input"
              id="code"
              type="text"
              required
              autoFocus={!!parentCode}
              placeholder={t("placeholders.code")}
              value={code}
              onChange={(e) => setCode(validateAccountCodeInput(e.target.value))}
            />
          </div>
          <div className="flex gap-2 w-full">
            <div className="w-full">
              <Label htmlFor="name">
                {t("fields.name")} <span className="text-destructive">*</span>
              </Label>
              <Input
                data-testid="chart-node-name-input"
                id="name"
                type="text"
                required
                placeholder={t("placeholders.name")}
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="w-full">
              <Label htmlFor="normalBalanceType">
                {t("fields.normalBalanceType")}{" "}
                <span className="text-destructive">*</span>
              </Label>
              <Select
                value={normalBalanceType}
                onValueChange={(value) => setNormalBalanceType(value as DebitOrCredit)}
              >
                <SelectTrigger data-testid="chart-node-balance-type-select">
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
          </div>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              disabled={loading || isNavigating}
              data-testid="chart-node-submit-button"
            >
              {t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
