"use client"

import { gql } from "@apollo/client"
import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { Switch } from "@lana/web/ui/switch"
import { FormEvent, useEffect, useState } from "react"

import {
  ExampleConfigurationRecord,
  ExampleConfigurationSetInput,
  useExampleConfigurationSetMutation,
  ExampleConfigurationDocument,
} from "@/lib/graphql/generated"

gql`
  mutation ExampleConfigurationSet($input: ExampleConfigurationSetInput!) {
    exampleConfigurationSet(input: $input) {
      exampleConfiguration {
        value {
          featureEnabled
          threshold
        }
        updatedBy
        updatedAt
        reason
        correlationId
      }
    }
  }
`

type ExampleConfigUpdateDialogProps = {
  open: boolean
  setOpen: (isOpen: boolean) => void
  exampleConfiguration?: ExampleConfigurationRecord | null
}

const initialFormData: ExampleConfigurationSetInput = {
  featureEnabled: false,
  threshold: 0,
  reason: null,
  correlationId: null,
}

export const ExampleConfigUpdateDialog: React.FC<ExampleConfigUpdateDialogProps> = ({
  open,
  setOpen,
  exampleConfiguration,
}) => {
  const [formData, setFormData] =
    useState<ExampleConfigurationSetInput>(initialFormData)

  const [saveConfig, { loading, error, reset }] = useExampleConfigurationSetMutation({
    refetchQueries: [ExampleConfigurationDocument],
  })

  useEffect(() => {
    if (exampleConfiguration?.value) {
      setFormData({
        featureEnabled: exampleConfiguration.value.featureEnabled,
        threshold: exampleConfiguration.value.threshold,
        reason: exampleConfiguration.reason ?? null,
        correlationId: exampleConfiguration.correlationId ?? null,
      })
    }
  }, [exampleConfiguration])

  const close = () => {
    reset()
    setOpen(false)
    setFormData(initialFormData)
  }

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await saveConfig({ variables: { input: formData } })
    close()
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Update Example Configuration</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit} className="space-y-4">
          <div className="flex items-center space-x-2">
            <Switch
              id="featureEnabled"
              checked={formData.featureEnabled}
              onCheckedChange={(checked) =>
                setFormData({ ...formData, featureEnabled: checked })
              }
            />
            <Label htmlFor="featureEnabled">Feature enabled</Label>
          </div>
          <div>
            <Label htmlFor="threshold">Threshold</Label>
            <Input
              id="threshold"
              type="number"
              value={formData.threshold}
              onChange={(e) =>
                setFormData({ ...formData, threshold: Number(e.target.value) })
              }
              required
            />
          </div>
          <div>
            <Label htmlFor="reason">Reason (optional)</Label>
            <Input
              id="reason"
              value={formData.reason ?? ""}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  reason: e.target.value || null,
                })
              }
            />
          </div>
          <div>
            <Label htmlFor="correlationId">Correlation ID (optional)</Label>
            <Input
              id="correlationId"
              value={formData.correlationId ?? ""}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  correlationId: e.target.value || null,
                })
              }
            />
          </div>
          {error && <div className="text-destructive">{error.message}</div>}
          <DialogFooter className="mt-4">
            <Button variant="outline" type="button" onClick={close}>
              Cancel
            </Button>
            <Button loading={loading} type="submit">
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
