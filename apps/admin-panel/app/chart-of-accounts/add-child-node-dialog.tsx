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

import { MAX_ACCOUNT_CODE_DIGITS } from "./constants"

import { useModalNavigation } from "@/hooks/use-modal-navigation"

import {
  useChartOfAccountsAddChildNodeMutation,
  useChartOfAccountsForLedgerQuery,
} from "@/lib/graphql/generated"

gql`
  query ChartOfAccountsForLedger {
    chartOfAccounts {
      id
      chartId
      name
    }
  }

  mutation ChartOfAccountsAddChildNode($input: ChartOfAccountsAddChildNodeInput!) {
    chartOfAccountsAddChildNode(input: $input) {
      chartOfAccounts {
        ...ChartOfAccountsFields
      }
    }
  }
`

type AddChildNodeDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  parentCode: string
  parentName?: string
}

export const AddChildNodeDialog: React.FC<AddChildNodeDialogProps> = ({
  open,
  onOpenChange,
  parentCode,
  parentName,
}) => {
  const t = useTranslations("ChartOfAccounts.AddChildNodeDialog")
  const [addChildNode, { loading }] = useChartOfAccountsAddChildNodeMutation()

  const { data: chartData } = useChartOfAccountsForLedgerQuery()

  const [code, setCode] = useState("")
  const [name, setName] = useState("")
  const [error, setError] = useState<string | null>(null)

  const { navigate } = useModalNavigation({
    closeModal: () => {
      resetForm()
      onOpenChange(false)
    },
  })

  const validateAccountCode = (value: string): string => {
    const cleaned = value.replace(/[^0-9.]/g, "")
    const maxChildDigits = MAX_ACCOUNT_CODE_DIGITS - parentCode.replace(/\./g, "").length

    if (maxChildDigits <= 0) return ""

    let result = ""
    let digitCount = 0

    for (const char of cleaned) {
      if (char === "." || digitCount < maxChildDigits) {
        result += char
        if (char !== ".") digitCount++
      } else break
    }

    return result
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const trimmedCode = code.trim()
    const trimmedName = name.trim()

    if (!trimmedCode || !trimmedName) {
      setError(t("errors.required"))
      return
    }

    if (!chartData?.chartOfAccounts?.chartId) {
      setError(t("errors.chartNotFound"))
      return
    }

    if (trimmedCode.replace(/\./g, "").length === 0) {
      setError(t("errors.childCodeRequired"))
      return
    }

    try {
      await addChildNode({
        variables: {
          input: {
            parent: parentCode,
            code: `${parentCode}.${trimmedCode}`,
            name: trimmedName,
          },
        },
      })

      const fullAccountCode = `${parentCode}.${trimmedCode}`
      toast.success(t("success"))
      resetForm()
      navigate(`/ledger-accounts/${fullAccountCode}`)
    } catch (error) {
      console.error("Error adding child node:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const resetForm = () => {
    setCode("")
    setName("")
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
          <DialogDescription>
            {t("description", { parentCode, parentName: parentName || "" })}
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="code">
              {t("fields.code")} <span className="text-destructive">*</span>
            </Label>
            <Input
              data-testid="child-node-code-input"
              id="code"
              type="text"
              required
              autoFocus
              placeholder={t("placeholders.code")}
              startAdornment={`${parentCode}.`}
              value={code}
              onChange={(e) => setCode(validateAccountCode(e.target.value))}
            />
            <p className="text-xs text-muted-foreground mt-1">
              {t("codeHint", {
                remaining:
                  MAX_ACCOUNT_CODE_DIGITS -
                  (parentCode.replace(/\./g, "").length + code.replace(/\./g, "").length),
                max: MAX_ACCOUNT_CODE_DIGITS,
              })}
            </p>
          </div>

          <div>
            <Label htmlFor="name">
              {t("fields.name")} <span className="text-destructive">*</span>
            </Label>
            <Input
              data-testid="child-node-name-input"
              id="name"
              type="text"
              required
              placeholder={t("placeholders.name")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
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
              data-testid="child-node-submit-button"
            >
              {loading ? t("buttons.adding") : t("buttons.add")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
