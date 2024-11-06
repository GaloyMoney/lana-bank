"use client"

import Link from "next/link"
import { FaExternalLinkAlt } from "react-icons/fa"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import Balance from "@/components/balance/balance"

import { GetCustomerQuery } from "@/lib/graphql/generated"
import { formatCollateralizationState } from "@/lib/utils"
import { LoanAndCreditFacilityStatusBadge } from "@/app/loans/status-badge"

type CustomerCreditFacilitiesTableProps = {
  creditFacilities: NonNullable<GetCustomerQuery["customer"]>["creditFacilities"]
}

export const CustomerCreditFacilitiesTable: React.FC<
  CustomerCreditFacilitiesTableProps
> = ({ creditFacilities }) => (
  <Card className="mt-4">
    <CardHeader>
      <CardTitle>Credit Facilities</CardTitle>
    </CardHeader>
    {creditFacilities.length === 0 ? (
      <CardContent>No credit facilities found for this customer</CardContent>
    ) : (
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Credit Facility ID</TableHead>
              <TableCell>Collateral (BTC)</TableCell>
              <TableHead>Outstanding Balance</TableHead>
              <TableHead>Collateralization State</TableHead>
              <TableCell>Status</TableCell>
              <TableHead>Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {creditFacilities.map((facility) => (
              <TableRow key={facility.creditFacilityId}>
                <TableCell>{facility.creditFacilityId}</TableCell>
                <TableCell>
                  <Balance
                    amount={facility.balance.collateral.btcBalance}
                    currency="btc"
                  />
                </TableCell>
                <TableCell>
                  <Balance
                    amount={facility.balance.outstanding.usdBalance}
                    currency="usd"
                  />
                </TableCell>
                <TableCell>
                  <LoanAndCreditFacilityStatusBadge status={facility.status} />
                </TableCell>
                <TableCell>
                  {formatCollateralizationState(facility.collateralizationState)}
                </TableCell>
                <TableCell>
                  <Link href={`/credit-facilities/${facility.creditFacilityId}`}>
                    <FaExternalLinkAlt className="text-primary" />
                  </Link>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    )}
  </Card>
)
