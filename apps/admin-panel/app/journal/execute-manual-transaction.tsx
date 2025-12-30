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

import { Plus, Trash2 } from "lucide-react"

import {
  DebitOrCredit,
  ManualTransactionEntryInput,
  ManualTransactionExecuteInput,
  useExecuteManualTransactionMutation,
} from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"
import DataTable from "@/components/data-table"
import { getCurrentLocalDate } from "@/lib/utils"

gql`
  mutation ExecuteManualTransaction($input: ManualTransactionExecuteInput!) {
    manualTransactionExecute(input: $input) {
      transaction {
        id
        ledgerTransactionId
        createdAt
        description
      }
    }
  }
`

const defaultEntry: ManualTransactionEntryInput = {
  accountRef: "",
  amount: 0,
  currency: "USD",
  direction: DebitOrCredit.Credit,
  description: "",
}

type ExecuteManualTransactionProps = {
  setOpenExecuteManualTransaction: (isOpen: boolean) => void
  openExecuteManualTransaction: boolean
}

export const ExecuteManualTransactionDialog: React.FC<ExecuteManualTransactionProps> = ({
  setOpenExecuteManualTransaction,
  openExecuteManualTransaction,
}) => {
  const t = useTranslations("ManualTransactions")
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => {
      setOpenExecuteManualTransaction(false)
      resetForm()
    },
  })

  const [
    executeManualTransaction,
    { loading, reset, error: executeManualTransactionError },
  ] = useExecuteManualTransactionMutation({})

  const isLoading = loading || isNavigating

  const [formValues, setFormValues] = useState<ManualTransactionExecuteInput>({
    description: "",
    reference: "",
    effective: getCurrentLocalDate(),
    entries: [defaultEntry, defaultEntry],
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
      await executeManualTransaction({
        variables: {
          input: { ...formValues },
        },
        onCompleted: (data) => {
          if (data?.manualTransactionExecute.transaction) {
            toast.success(t("success"))
            navigate(
              `/ledger-transactions/${data.manualTransactionExecute.transaction.ledgerTransactionId}`,
            )
          } else {
            throw new Error(t("errored"))
          }
        },
      })
    } catch (error) {
      console.error("Error executing manual transaction:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (executeManualTransactionError) {
        setError(executeManualTransactionError.message)
      } else {
        setError(t("errored"))
      }
      toast.error(t("errored"))
    }
  }

  const resetForm = () => {
    setFormValues({
      description: "",
      reference: "",
      effective: getCurrentLocalDate(),
      entries: [defaultEntry, defaultEntry],
    })
    setError(null)
    reset()
  }

  const addJournalEntryRow = () => {
    setFormValues((prevValues) => ({
      ...prevValues,
      entries: [...prevValues.entries, defaultEntry],
    }))
  }

  const deleteJournalEntryRow = (index: number) => {
    setFormValues((prevValues) => ({
      ...prevValues,
      entries: prevValues.entries.filter((_, i) => i !== index),
    }))
  }

  return (
    <Dialog
      open={openExecuteManualTransaction}
      onOpenChange={(isOpen) => {
        setOpenExecuteManualTransaction(isOpen)
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogContent className="sm:min-w-[1024px]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="description">{t("fields.description")}</Label>
            <Input
              id="description"
              name="description"
              type="text"
              required
              placeholder={t("placeholders.description")}
              value={formValues.description}
              onChange={handleChange}
              disabled={isLoading}
              data-testid="execute-manual-transaction-description-input"
            />
          </div>
          <div className="flex w-full gap-2">
            <div className="w-1/2">
              <Label htmlFor="reference">{t("fields.reference")}</Label>
              <Input
                id="reference"
                name="reference"
                type="text"
                required
                placeholder={t("placeholders.reference")}
                value={formValues.reference || ""}
                onChange={handleChange}
                disabled={isLoading}
                data-testid="execute-manual-transaction-description-input"
              />
            </div>
            <div className="w-1/2">
              <Label htmlFor="effective">{t("fields.effective")}</Label>
              <Input
                required
                id="effective"
                name="effective"
                type="date"
                value={formValues.effective}
                onChange={handleChange}
                disabled={isLoading}
                data-testid="execute-manual-transaction-effective-date-input"
              />
            </div>
          </div>
          <div>
            <div className="flex justify-between items-center w-full mb-2">
              <Label htmlFor="entries">{t("fields.entries")}</Label>
              <Button
                size="sm"
                type="button"
                variant="outline"
                onClick={addJournalEntryRow}
                disabled={isLoading}
                data-testid="execute-manual-transaction-entry-input-button"
              >
                <Plus />
                {t("addEntryBtn")}
              </Button>
            </div>
            <DataTable
              data={formValues.entries}
              emptyMessage={t("noEntries")}
              columns={[
                {
                  key: "accountRef",
                  header: t("table.accountRef"),
                  render: (_, __, index) => (
                    <Input
                      type="text"
                      required
                      placeholder={t("placeholdersJournalEntry.accountRef")}
                      value={formValues.entries[index].accountRef}
                      onChange={(e) => {
                        const value = e.target.value
                        setFormValues((prevValues) => ({
                          ...prevValues,
                          entries: [
                            ...prevValues.entries.slice(0, index),
                            { ...prevValues.entries[index], accountRef: value },
                            ...prevValues.entries.slice(index + 1),
                          ],
                        }))
                      }}
                    />
                  ),
                },
                {
                  key: "amount",
                  header: t("table.amount"),
                  render: (_, __, index) => (
                    <Input
                      type="number"
                      required
                      placeholder={t("placeholdersJournalEntry.amount")}
                      value={formValues.entries[index].amount}
                      onChange={(e) => {
                        const value = e.target.value
                        setFormValues((prevValues) => ({
                          ...prevValues,
                          entries: [
                            ...prevValues.entries.slice(0, index),
                            { ...prevValues.entries[index], amount: value },
                            ...prevValues.entries.slice(index + 1),
                          ],
                        }))
                      }}
                    />
                  ),
                },
                {
                  key: "currency",
                  header: t("table.currency"),
                  render: (_, __, index) => (
                    <Select
                      value={formValues.entries[index].currency}
                      onValueChange={(value) => {
                        setFormValues((prevValues) => ({
                          ...prevValues,
                          entries: [
                            ...prevValues.entries.slice(0, index),
                            { ...prevValues.entries[index], currency: value },
                            ...prevValues.entries.slice(index + 1),
                          ],
                        }))
                      }}
                    >
                      <SelectTrigger
                        id={`currency${index}`}
                        data-testid={`currency${index}`}
                      >
                        <SelectValue
                          placeholder={t("placeholdersJournalEntry.currency")}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value={"USD"}>{t("fields.usd")}</SelectItem>
                        <SelectItem value={"BTC"}>{t("fields.btc")}</SelectItem>
                      </SelectContent>
                    </Select>
                  ),
                },
                {
                  key: "direction",
                  header: t("table.direction"),
                  render: (_, __, index) => (
                    <Select
                      value={formValues.entries[index].direction}
                      onValueChange={(value) => {
                        setFormValues((prevValues) => ({
                          ...prevValues,
                          entries: [
                            ...prevValues.entries.slice(0, index),
                            {
                              ...prevValues.entries[index],
                              direction: value as DebitOrCredit,
                            },
                            ...prevValues.entries.slice(index + 1),
                          ],
                        }))
                      }}
                    >
                      <SelectTrigger
                        id={`direction${index}`}
                        data-testid={`direction${index}`}
                      >
                        <SelectValue
                          placeholder={t("placeholdersJournalEntry.direction")}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value={DebitOrCredit.Debit}>
                          {t("fields.debit")}
                        </SelectItem>
                        <SelectItem value={DebitOrCredit.Credit}>
                          {t("fields.credit")}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  ),
                },
                {
                  key: "description",
                  header: t("table.description"),
                  render: (_, __, index) => (
                    <Input
                      type="text"
                      placeholder={t("placeholdersJournalEntry.description")}
                      value={formValues.entries[index].description}
                      onChange={(e) => {
                        const value = e.target.value
                        setFormValues((prevValues) => ({
                          ...prevValues,
                          entries: [
                            ...prevValues.entries.slice(0, index),
                            { ...prevValues.entries[index], description: value },
                            ...prevValues.entries.slice(index + 1),
                          ],
                        }))
                      }}
                    />
                  ),
                },
                {
                  key: "accountRef",
                  header: "",
                  render: (_, __, index) => (
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      disabled={isLoading}
                      className="bg-red-500 hover:bg-red-600 text-white"
                      onClick={() => deleteJournalEntryRow(index)}
                    >
                      <Trash2 />
                    </Button>
                  ),
                },
              ]}
            />
          </div>

          {error && <p className="text-destructive">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              data-testid="execute-manual-transaction-submit-button"
            >
              {t("execute")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
