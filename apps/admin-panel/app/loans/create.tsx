import { gql } from "@apollo/client"
import React, { useEffect, useState } from "react"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/primitive/dialog"
import { Input } from "@/components/primitive/input"
import { Label } from "@/components/primitive/label"
import {
  InterestInterval,
  Period,
  useDefaultTermsQuery,
  useGetRealtimePriceUpdatesQuery,
  useLoanCreateMutation,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { Select } from "@/components/primitive/select"
import { formatInterval, formatPeriod, currencyConverter } from "@/lib/utils"
import { DetailItem } from "@/components/details"
import Balance from "@/components/balance/balance"

gql`
  mutation LoanCreate($input: LoanCreateInput!) {
    loanCreate(input: $input) {
      loan {
        id
        loanId
        createdAt
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
          interestIncurred {
            usdBalance
          }
        }
        loanTerms {
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

type CreateLoanDialogProps = {
  customerId: string
  refetch?: () => void
}

export const CreateLoanDialog: React.FC<
  React.PropsWithChildren<CreateLoanDialogProps>
> = ({ customerId, children, refetch }) => {
  const router = useRouter()

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const [customerIdValue, setCustomerIdValue] = useState<string>(customerId)
  const { data: defaultTermsData } = useDefaultTermsQuery()
  const [createLoan, { loading, error, reset }] = useLoanCreateMutation()
  const [useDefaultTerms, setUseDefaultTerms] = useState(true)

  useEffect(() => {
    if (!defaultTermsData) setUseDefaultTerms(false)
  }, [defaultTermsData, setUseDefaultTerms])

  const [formValues, setFormValues] = useState({
    desiredPrincipal: "0",
    annualRate: "",
    interval: "",
    liquidationCvl: "",
    marginCallCvl: "",
    initialCvl: "",
    durationUnits: "",
    durationPeriod: "",
  })

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target
    setFormValues((prevValues) => ({
      ...prevValues,
      [name]: value,
    }))
  }

  const handleCreateLoan = async (event: React.FormEvent) => {
    event.preventDefault()
    const {
      desiredPrincipal,
      annualRate,
      interval,
      liquidationCvl,
      marginCallCvl,
      initialCvl,
      durationUnits,
      durationPeriod,
    } = formValues

    if (
      !desiredPrincipal ||
      !annualRate ||
      !interval ||
      !liquidationCvl ||
      !marginCallCvl ||
      !initialCvl ||
      !durationUnits ||
      !durationPeriod
    ) {
      toast.error("Please fill in all the fields.")
      return
    }

    try {
      await createLoan({
        variables: {
          input: {
            customerId: customerIdValue,
            desiredPrincipal: currencyConverter.usdToCents(Number(desiredPrincipal)),
            loanTerms: {
              annualRate: parseFloat(annualRate),
              interval: interval as InterestInterval,
              liquidationCvl: parseFloat(liquidationCvl),
              marginCallCvl: parseFloat(marginCallCvl),
              initialCvl: parseFloat(initialCvl),
              duration: {
                units: parseInt(durationUnits),
                period: durationPeriod as Period,
              },
            },
          },
        },
        onCompleted: (data) => {
          toast.success("Loan created successfully")
          router.push(`/loans/${data?.loanCreate.loan.loanId}`)
        },
      })

      if (refetch) refetch()
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    setUseDefaultTerms(!!defaultTermsData?.defaultTerms?.id)
    if (defaultTermsData && defaultTermsData.defaultTerms) {
      const terms = defaultTermsData.defaultTerms.values
      setFormValues({
        desiredPrincipal: "0",
        annualRate: terms.annualRate.toString(),
        interval: terms.interval,
        liquidationCvl: terms.liquidationCvl.toString(),
        marginCallCvl: terms.marginCallCvl.toString(),
        initialCvl: terms.initialCvl.toString(),
        durationUnits: terms.duration.units.toString(),
        durationPeriod: terms.duration.period,
      })
    } else {
      setFormValues({
        desiredPrincipal: "0",
        annualRate: "",
        interval: "",
        liquidationCvl: "",
        marginCallCvl: "",
        initialCvl: "",
        durationUnits: "",
        durationPeriod: "",
      })
    }
  }

  const collateralRequiredForDesiredPrincipal = currencyConverter.btcToSatoshi(
    currencyConverter.usdToCents(Number(formValues.desiredPrincipal || 0)) /
      priceInfo?.realtimePrice.usdCentsPerBtc,
  )

  return (
    <Dialog
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          setCustomerIdValue(customerId)
          reset()
        }
        if (isOpen) {
          resetForm()
        }
      }}
    >
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent className="min-w-max">
        <DialogHeader>
          <DialogTitle>Create Loan</DialogTitle>
          <DialogDescription>Fill in the details to create a loan.</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleCreateLoan}>
          <div>
            <Label>Principal</Label>
            <div className="flex items-center gap-1">
              <Input
                type="number"
                name="desiredPrincipal"
                value={formValues.desiredPrincipal}
                onChange={handleChange}
                placeholder="Enter the desired principal amount"
                min={0}
                required
              />
              <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
            </div>
            {priceInfo && (
              <div className="mt-2 text-sm flex space-x-1 items-center">
                <Balance amount={collateralRequiredForDesiredPrincipal} currency="btc" />
                <div>collateral required (</div>
                <div>BTC/USD: </div>
                {
                  <Balance
                    amount={priceInfo?.realtimePrice.usdCentsPerBtc}
                    currency="usd"
                  />
                }
                <div>)</div>
              </div>
            )}
          </div>
          {useDefaultTerms ? (
            <>
              <div
                onClick={() => setUseDefaultTerms(false)}
                className="mt-2 flex items-center space-x-2 ml-2 cursor-pointer text-sm hover:underline w-fit"
              >
                <div>Loan Terms</div>
                <PiPencilSimpleLineLight className="w-5 h-5 cursor-pointer" />
              </div>
              <div className="grid grid-cols-2 gap-x-2">
                <DetailItem
                  label="Interest Rate (APR)"
                  value={formValues.annualRate + "%"}
                />
                <DetailItem label="Initial CVL %" value={formValues.initialCvl} />
                <DetailItem
                  label="Duration"
                  value={
                    String(formValues.durationUnits) +
                    " " +
                    formatPeriod(formValues.durationPeriod as Period)
                  }
                />
                <DetailItem label="Margin Call CVL %" value={formValues.marginCallCvl} />
                <DetailItem
                  label="Payment Schedule"
                  className="space-x-7"
                  value={formatInterval(formValues.interval as InterestInterval)}
                />
                <DetailItem label="Liquidation CVL %" value={formValues.liquidationCvl} />
              </div>
            </>
          ) : (
            <>
              <div>
                <Label>Initial CVL (%)</Label>
                <Input
                  type="number"
                  name="initialCvl"
                  value={formValues.initialCvl}
                  onChange={handleChange}
                  placeholder="Enter the initial CVL"
                  required
                />
              </div>
              <div>
                <Label>Margin Call CVL (%)</Label>
                <Input
                  type="number"
                  name="marginCallCvl"
                  value={formValues.marginCallCvl}
                  onChange={handleChange}
                  placeholder="Enter the margin call CVL"
                  required
                />
              </div>

              <div>
                <Label>Liquidation CVL (%)</Label>
                <Input
                  type="number"
                  name="liquidationCvl"
                  value={formValues.liquidationCvl}
                  onChange={handleChange}
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
                    name="durationUnits"
                    value={formValues.durationUnits}
                    onChange={handleChange}
                    placeholder="Duration"
                    min={0}
                    required
                    className="w-1/2"
                  />
                  <Select
                    name="durationPeriod"
                    value={formValues.durationPeriod}
                    onChange={handleChange}
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
                <Label>Interest Payment Schedule</Label>
                <Select
                  name="interval"
                  value={formValues.interval}
                  onChange={handleChange}
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
                <Label>Annual Rate (%)</Label>
                <Input
                  type="number"
                  name="annualRate"
                  value={formValues.annualRate}
                  onChange={handleChange}
                  placeholder="Enter the annual rate"
                  required
                />
              </div>
            </>
          )}
          {error && <span className="text-destructive">{error.message}</span>}
          <DialogFooter className={!useDefaultTerms ? "sm:justify-between" : ""}>
            {!useDefaultTerms && (
              <div
                onClick={() => setUseDefaultTerms(true)}
                className="flex items-center space-x-2 cursor-pointer text-sm hover:underline"
              >
                Show less...
              </div>
            )}
            <Button
              onClick={handleCreateLoan}
              className="w-32"
              disabled={loading}
              type="submit"
              loading={loading}
            >
              Create New Loan
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
