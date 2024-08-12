import { gql } from "@apollo/client"

gql`
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
