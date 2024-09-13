import React from "react"

import { Bar } from "react-chartjs-2"
import {
  Chart,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  ChartData,
} from "chart.js"

Chart.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
)

import { GetLoanDetailsQuery } from "@/lib/graphql/generated"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { currencyConverter, formatDate } from "@/lib/utils"
import Balance from "@/components/balance/balance"

type RepaymentPlanProps = {
  loan: NonNullable<GetLoanDetailsQuery["loan"]>
}
export const RepaymentPlan: React.FC<RepaymentPlanProps> = ({ loan }) => {
  return (
    <>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Repayment Plan</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Type</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Accural At</TableHead>
                <TableHead>Due At</TableHead>
                <TableHead>
                  <span className="flex justify-end">Initial</span>
                </TableHead>
                <TableHead>
                  <span className="flex justify-end">Outstanding</span>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loan.repaymentPlan.map((repayment, index) => (
                <TableRow key={index}>
                  <TableCell>{repayment.repaymentType}</TableCell>
                  <TableCell>{repayment.status}</TableCell>
                  <TableCell>{formatDate(repayment.accrualAt)}</TableCell>
                  <TableCell>{formatDate(repayment.dueAt)}</TableCell>
                  <TableCell>
                    <Balance
                      className="flex justify-end"
                      amount={repayment.initial}
                      currency="usd"
                    />
                  </TableCell>
                  <TableCell>
                    <Balance
                      className="flex justify-end"
                      amount={repayment.outstanding}
                      currency="usd"
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      <LoanAmortizationGraph loan={loan} />
    </>
  )
}

const LoanAmortizationGraph: React.FC<RepaymentPlanProps> = ({
  loan: { repaymentPlan: repaymentPlanData, facility },
}) => {
  const repaymentPlan = repaymentPlanData.map((r) => ({
    ...r,
    initial: currencyConverter.centsToUsd(r.initial),
    outstanding: currencyConverter.centsToUsd(r.outstanding),
  }))

  const processRepaymentData = () => {
    let outstandingBalance = facility

    const labels: string[] = []
    const principalPaid: (number | null)[] = []
    const interestPaid: (number | null)[] = []
    const outstandingPrincipal: number[] = []

    repaymentPlan.forEach((payment) => {
      labels.push(new Date(payment.dueAt).toLocaleDateString())

      if (payment.repaymentType === "DISBURSEMENTS") {
        if (payment.status === "PAID") {
          outstandingBalance -= payment.initial
        }
        principalPaid.push(payment.initial)
        outstandingPrincipal.push(outstandingBalance)
      } else {
        principalPaid.push(null) // Push null to avoid rendering a bar for zero values
        outstandingPrincipal.push(outstandingBalance)
      }

      if (payment.repaymentType === "INTEREST") {
        interestPaid.push(payment.initial)
      } else {
        interestPaid.push(null) // Push null to avoid rendering a bar for zero values
      }
    })

    return { labels, principalPaid, interestPaid, outstandingPrincipal }
  }

  const { labels, principalPaid, interestPaid, outstandingPrincipal } =
    processRepaymentData()

  const data = {
    labels: labels,
    datasets: [
      {
        type: "bar" as const,
        label: "Principal Paid",
        data: principalPaid,
        backgroundColor: "rgb(75, 192, 192)",
        yAxisID: "y1",
      },
      {
        type: "bar" as const,
        label: "Interest Paid",
        data: interestPaid,
        backgroundColor: "rgb(255, 99, 132)",
        yAxisID: "y1",
      },
      {
        type: "line" as const,
        label: "Outstanding Principal",
        data: outstandingPrincipal,
        borderColor: "rgb(255, 159, 64)",
        fill: false,
        yAxisID: "y2",
        tension: 0.1,
      },
    ],
  }

  const options = {
    scales: {
      y1: {
        type: "linear" as const,
        position: "left" as const,
        title: {
          display: true,
          text: "Payments (USD)",
        },
      },
      y2: {
        type: "linear" as const,
        position: "right" as const,
        title: {
          display: true,
          text: "Outstanding Balance (USD)",
        },
        grid: {
          drawOnChartArea: false,
        },
      },
    },
  }

  console.log({ chartData: JSON.stringify(data) })

  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Loan Amortization Graph</CardTitle>
      </CardHeader>
      <CardContent className="flex justify-center">
        <Bar
          className="p-10"
          data={data as ChartData<"bar", number[], string>}
          options={options}
        />
      </CardContent>
    </Card>
  )
}
