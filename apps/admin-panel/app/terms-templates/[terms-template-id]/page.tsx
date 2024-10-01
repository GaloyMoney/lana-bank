import TermsTemplateDetails from "./details"

import { PageHeading } from "@/components/page-heading"

function TermsTemplate({
  params,
}: {
  params: {
    "terms-template-id": string
  }
}) {
  const { "terms-template-id": termsTemplateId } = params

  return (
    <main>
      <PageHeading>Terms Template Details</PageHeading>
      <TermsTemplateDetails id={termsTemplateId} />
    </main>
  )
}

export default TermsTemplate
