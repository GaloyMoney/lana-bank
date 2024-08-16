import { gql } from "@apollo/client"
import { useState } from "react"
import { toast } from "sonner"

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
  useLoanCreateMutation,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { currencyConverter, formatDate } from "@/lib/utils"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Select } from "@/components/primitive/select"
import { formatInterval, formatPeriod } from "@/lib/terms/utils"
import Balance from "@/components/balance/balance"
import { useRouter } from "next/navigation"

gql`
  mutation LoanCreate($input: LoanCreateInput!) {
    loanCreate(input: $input) {
      loan {
        id
        loanId
        startDate
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

export const CreateLoanDialog = ({
  customerId,
  children,
  refetch,
}: {
  customerId: string
  children: React.ReactNode
  refetch?: () => void
}) => {
  const router = useRouter()

  const [customerIdValue, setCustomerIdValue] = useState<string>(customerId)
  const { data: defaultTermsData } = useDefaultTermsQuery()
  const [createLoan, { data, loading, error, reset }] = useLoanCreateMutation()

  const [formValues, setFormValues] = useState({
    desiredPrincipal: "",
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
      })
      toast.success("Loan created successfully")
      if (refetch) refetch()
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    if (defaultTermsData && defaultTermsData.defaultTerms) {
      const terms = defaultTermsData.defaultTerms.values
      setFormValues({
        desiredPrincipal: "",
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
        desiredPrincipal: "",
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
      {data ? (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Loan Created</DialogTitle>
            <DialogDescription>Loan Details</DialogDescription>
          </DialogHeader>
          <DetailsGroup>
            <DetailItem label="Loan ID" value={data.loanCreate.loan.loanId} />
            <DetailItem
              label="Start Date"
              value={formatDate(data.loanCreate.loan.startDate)}
            />
            <DetailItem
              label="Collateral"
              valueComponent={
                <Balance
                  amount={data.loanCreate.loan.balance.collateral.btcBalance}
                  currency="btc"
                />
              }
            />
            <DetailItem
              label="Interest Incurred"
              valueComponent={
                <Balance
                  amount={data.loanCreate.loan.balance.interestIncurred.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Outstanding"
              valueComponent={
                <Balance
                  amount={data.loanCreate.loan.balance.outstanding.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Duration"
              value={`${String(data.loanCreate.loan.loanTerms.duration.units)} ${formatPeriod(data.loanCreate.loan.loanTerms.duration.period)}`}
            />
            <DetailItem
              label="Interest Payment Schedule"
              value={formatInterval(data.loanCreate.loan.loanTerms.interval)}
            />
            <DetailItem
              label="Annual Rate"
              value={`${data.loanCreate.loan.loanTerms.annualRate}%`}
            />
            <DetailItem
              label="Initial CVL"
              value={`${data.loanCreate.loan.loanTerms.initialCvl}%`}
            />
            <DetailItem
              label="Margin Call CVL"
              value={`${data.loanCreate.loan.loanTerms.marginCallCvl}%`}
            />
            <DetailItem
              label="Liquidation CVL"
              value={`${data.loanCreate.loan.loanTerms.liquidationCvl}%`}
            />
          </DetailsGroup>
          <Button onClick={() => router.push(`/loan/${data.loanCreate.loan.loanId}`)}>
            View Details
          </Button>
        </DialogContent>
      ) : (
        <DialogContent>
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
            </div>
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
            {error && <span className="text-destructive">{error.message}</span>}
            <DialogFooter className="mt-4">
              <Button
                onClick={handleCreateLoan}
                className="w-32"
                disabled={loading}
                type="submit"
              >
                Create New Loan
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      )}
    </Dialog>
  )
}
