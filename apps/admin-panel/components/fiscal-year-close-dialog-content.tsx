import { useId, type ReactNode } from "react"

import { DialogHeader, DialogTitle } from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { Alert, AlertDescription } from "@lana/web/ui/alert"

import { DetailItem } from "@/components/details/item"
import { DetailsGroup } from "@/components/details/group"

type FiscalYearCloseDialogContentProps = {
  title: string
  content: {
    description: string
    warning: string
    closingLabel: string
    closingValue: string | null
    emptyStateMessage?: string
  }
  confirmation: {
    label: ReactNode
    expectedValue: string | null
    placeholder?: string
    value: string
    onChange: (value: string) => void
  }
  state: {
    error: string | null
    loading: boolean
  }
}

export function FiscalYearCloseDialogContent({
  title,
  content,
  confirmation,
  state,
}: FiscalYearCloseDialogContentProps) {
  const inputId = useId()
  const hasRequiredData = content.closingValue && confirmation.expectedValue
  return (
    <>
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
      </DialogHeader>
      {hasRequiredData ? (
        <FiscalYearCloseForm
          inputId={inputId}
          content={content}
          confirmation={confirmation}
          loading={state.loading}
        />
      ) : (
        <Alert variant="default">
          <AlertDescription>{content.emptyStateMessage}</AlertDescription>
        </Alert>
      )}
      {state.error && (
        <Alert variant="destructive">
          <AlertDescription>{state.error}</AlertDescription>
        </Alert>
      )}
    </>
  )
}

type FiscalYearCloseFormProps = {
  inputId: string
  content: FiscalYearCloseDialogContentProps["content"]
  confirmation: FiscalYearCloseDialogContentProps["confirmation"]
  loading: boolean
}

function FiscalYearCloseForm({
  inputId,
  content,
  confirmation,
  loading,
}: FiscalYearCloseFormProps) {
  return (
    <>
      <p className="text-sm text-muted-foreground">{content.description}</p>

      <Alert variant="warning">
        <AlertDescription className="font-medium">{content.warning}</AlertDescription>
      </Alert>

      <DetailsGroup layout="horizontal">
        <DetailItem
          label={content.closingLabel}
          value={content.closingValue}
          className="bg-muted/50 border rounded-lg p-2 text-sm"
        />
      </DetailsGroup>
      <div>
        <Label htmlFor={inputId} className="text-muted-foreground">
          {confirmation.label}
        </Label>
        <Input
          id={inputId}
          placeholder={confirmation.placeholder ?? confirmation.expectedValue ?? ""}
          value={confirmation.value}
          onChange={(e) => confirmation.onChange(e.target.value)}
          autoComplete="off"
          autoFocus
          disabled={loading}
        />
      </div>
    </>
  )
}
