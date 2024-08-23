"use client"

import { gql } from "@apollo/client"

import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"

import { Separator } from "@/components/primitive/separator"
import {
  useGetCustomerByCustomerIdQuery,
  useSumsubPermalinkCreateMutation,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"

gql`
  mutation sumsubPermalinkCreate($input: SumsubPermalinkCreateInput!) {
    sumsubPermalinkCreate(input: $input) {
      url
    }
  }

  query getCustomerByCustomerId($id: UUID!) {
    customer(id: $id) {
      customerId
      email
      status
      level
      applicantId
      balance {
        checking {
          settled
          pending
        }
      }
    }
  }
`

export const CustomerDetailsCard = ({ customerId }: { customerId: string }) => {
  const {
    loading,
    error,
    data: customerDetails,
  } = useGetCustomerByCustomerIdQuery({
    variables: {
      id: customerId,
    },
  })

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${customerDetails?.customer?.applicantId}/client/basicInfo`

  const [createLink, { data: linkData, loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation()

  const handleCreateLink = async () => {
    if (customerDetails?.customer?.customerId) {
      await createLink({
        variables: {
          input: {
            customerId: customerDetails.customer.customerId,
          },
        },
      })
    }
  }

  return (
    <Card>
      {loading ? (
        <CardContent className="p-6">Loading...</CardContent>
      ) : error ? (
        <CardContent className="p-6 text-destructive">{error.message}</CardContent>
      ) : !customerDetails || !customerDetails.customer ? (
        <CardContent className="p-6">No customer found with this ID</CardContent>
      ) : (
        <>
          <CardHeader className="pb-4">
            <div className="flex justify-between items-center">
              <CardTitle>
                <div className="flex flex-col gap-1">
                  <p className="text-lg">{customerDetails.customer.email}</p>
                </div>
              </CardTitle>
            </div>
          </CardHeader>
          <Separator />
          <Card className="mt-4" variant="transparent">
            <CardContent>
              <DetailsGroup>
                <DetailItem
                  label="Customer ID"
                  value={customerDetails.customer.customerId}
                />
                <DetailItem label="Email" value={customerDetails.customer.email} />
                <DetailItem label="Status" value={customerDetails.customer.status} />
                <DetailItem
                  label="Applicant Id"
                  value={customerDetails.customer.applicantId ?? "not yet connected"}
                  valueComponent={
                    customerDetails.customer.applicantId ? (
                      <a
                        href={sumsubLink}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-500 underline"
                      >
                        {customerDetails.customer.applicantId}
                      </a>
                    ) : (
                      <div>
                        <p>not set yet</p>
                        {!linkData && (
                          <button
                            onClick={handleCreateLink}
                            className="text-blue-500 underline"
                            disabled={linkLoading}
                          >
                            {linkLoading ? "Creating link..." : "Create Link"}
                          </button>
                        )}
                        {linkData && linkData.sumsubPermalinkCreate && (
                          <a
                            href={linkData.sumsubPermalinkCreate.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-500 underline"
                          >
                            {linkData.sumsubPermalinkCreate.url}
                          </a>
                        )}
                        {linkError && <p className="text-red-500">{linkError.message}</p>}
                      </div>
                    )
                  }
                />
                <DetailItem label="Level" value={customerDetails.customer.level} />
                <DetailItem
                  label="Checking Settled Balance (USD)"
                  valueComponent={
                    <Balance
                      amount={customerDetails.customer.balance.checking.settled}
                      currency="usd"
                    />
                  }
                />
                <DetailItem
                  label="Pending withdrawals (USD)"
                  valueComponent={
                    <Balance
                      amount={customerDetails.customer.balance.checking.pending}
                      currency="usd"
                    />
                  }
                />
              </DetailsGroup>
            </CardContent>
          </Card>
        </>
      )}
    </Card>
  )
}
