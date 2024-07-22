"use client"

import { gql } from "@apollo/client"
import React, { useState } from "react"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../primitive/dialog"
import { DetailItem, DetailsGroup } from "../details"
import { Label } from "../primitive/label"
import { Input } from "../primitive/input"
import { Select } from "../primitive/select"
import { Button } from "../primitive/button"

import {
  InterestInterval,
  Period,
  useCurrentTermsUpdateMutation,
} from "@/lib/graphql/generated"
import { formatInterval, formatPeriod } from "@/lib/term/utils"

gql`
  mutation CurrentTermsUpdate($input: CurrentTermsUpdateInput!) {
    currentTermsUpdate(input: $input) {
      terms {
        id
        termsId
        values {
          annualRate
          interval
          liquidationCvl
          marginCallCvl
          initialCvl
          duration {
            period
            units
          }
        }
      }
    }
  }
`

export const UpdateCurrentTermDialog: React.FC<{
  children: React.ReactNode
  refetch?: () => void
}> = ({ children, refetch }) => {
  const [interval, setInterval] = useState<InterestInterval | "">("")
  const [liquidationCvl, setLiquidationCvl] = useState<string>("")
  const [marginCallCvl, setMarginCallCvl] = useState<string>("")
  const [initialCvl, setInitialCvl] = useState<string>("")
  const [duration, setDuration] = useState<{ period: Period | ""; units: number | "" }>({
    period: "",
    units: "",
  })
  const [annualRate, setAnnualRate] = useState<number | "">("")

  const [updateCurrentTerm, { data, loading, error, reset }] =
    useCurrentTermsUpdateMutation()

  const handleUpdateCurrentTerm = async (event: React.FormEvent) => {
    event.preventDefault()
    console.log(annualRate, interval, duration, liquidationCvl, marginCallCvl, initialCvl)

    if (
      annualRate === "" ||
      interval === "" ||
      duration.period === "" ||
      duration.units === "" ||
      liquidationCvl === "" ||
      marginCallCvl === "" ||
      initialCvl === ""
    ) {
      toast.error("Please fill in all the fields")
      return
    }

    try {
      await updateCurrentTerm({
        variables: {
          input: {
            annualRate: annualRate,
            interval: interval as InterestInterval,
            duration: {
              period: duration.period as Period,
              units: Number(duration.units),
            },
            liquidationCvl,
            marginCallCvl,
            initialCvl,
          },
        },
      })
      toast.success("Current term updated")
      if (refetch) refetch()
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    setInterval("")
    setLiquidationCvl("")
    setMarginCallCvl("")
    setInitialCvl("")
    setDuration({ period: "", units: "" })
    setAnnualRate("")
    reset()
  }

  return (
    <Dialog
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogTrigger asChild>{children}</DialogTrigger>
      {data ? (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Terms Updated</DialogTitle>
            <DialogDescription>Terms Details.</DialogDescription>
          </DialogHeader>
          <DetailsGroup>
            <DetailItem label="Terms ID" value={data.currentTermsUpdate.terms.termsId} />
            <DetailItem
              label="Duration"
              value={
                String(data.currentTermsUpdate.terms.values.duration.units) +
                " " +
                formatPeriod(data.currentTermsUpdate.terms.values.duration.period)
              }
            />
            <DetailItem
              label="Interval"
              value={formatInterval(data.currentTermsUpdate.terms.values.interval)}
            />
            <DetailItem
              label="Annual Rate"
              value={data.currentTermsUpdate.terms.values.annualRate}
            />
            <DetailItem
              label="Liquidation CVL"
              value={data.currentTermsUpdate.terms.values.liquidationCvl}
            />
            <DetailItem
              label="Margin Call CVL"
              value={data.currentTermsUpdate.terms.values.marginCallCvl}
            />
            <DetailItem
              label="Initial CVL"
              value={data.currentTermsUpdate.terms.values.initialCvl}
            />
          </DetailsGroup>
        </DialogContent>
      ) : (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Update Terms</DialogTitle>
            <DialogDescription>
              Fill in the details to update the terms.
            </DialogDescription>
          </DialogHeader>
          <form className="flex flex-col gap-4" onSubmit={handleUpdateCurrentTerm}>
            <div>
              <Label>Margin Call CVL</Label>
              <Input
                type="number"
                value={marginCallCvl}
                onChange={(e) => setMarginCallCvl(e.target.value)}
                placeholder="Enter the margin call CVL"
                required
              />
            </div>
            <div>
              <Label>Initial CVL</Label>
              <Input
                type="number"
                value={initialCvl}
                onChange={(e) => setInitialCvl(e.target.value)}
                placeholder="Enter the initial CVL"
                required
              />
            </div>
            <div>
              <Label>Liquidation CVL</Label>
              <Input
                type="number"
                value={liquidationCvl}
                onChange={(e) => setLiquidationCvl(e.target.value)}
                placeholder="Enter the liquidation CVL"
                min={0}
                required
              />
            </div>
            <div>
              <Label>Duration</Label>
              <div className="flex gap-2">
                <Input
                  type="number"
                  value={duration.units}
                  onChange={(e) =>
                    setDuration({
                      ...duration,
                      units: e.target.value === "" ? "" : parseInt(e.target.value),
                    })
                  }
                  placeholder="Duration"
                  min={0}
                  required
                  className="w-1/2"
                />
                <Select
                  value={duration.period}
                  onChange={(e) =>
                    setDuration({ ...duration, period: e.target.value as Period })
                  }
                  required
                >
                  <option value="" disabled>
                    Select period
                  </option>
                  {Object.values(Period).map((period) => (
                    <option key={period} value={period}>
                      {formatPeriod(period)}
                    </option>
                  ))}
                </Select>
              </div>
            </div>
            <div>
              <Label>Interval</Label>
              <Select
                value={interval}
                onChange={(e) => setInterval(e.target.value as InterestInterval)}
                required
              >
                <option value="" disabled>
                  Select interval
                </option>
                {Object.values(InterestInterval).map((interval) => (
                  <option key={interval} value={interval}>
                    {formatInterval(interval)}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>Annual Rate</Label>
              <Input
                type="number"
                value={annualRate}
                onChange={(e) =>
                  setAnnualRate(e.target.value === "" ? "" : parseFloat(e.target.value))
                }
                placeholder="Enter the annual rate"
                required
              />
            </div>
            {error && <span className="text-destructive">{error.message}</span>}
            <DialogFooter className="mt-4">
              <Button className="w-32" type="submit" loading={loading}>
                Submit
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      )}
    </Dialog>
  )
}
