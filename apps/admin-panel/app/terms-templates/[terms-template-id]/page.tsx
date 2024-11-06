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
    <main className="max-w-[70rem] m-auto">
      <PageHeading>Terms Template Details</PageHeading>
      <TermsTemplateDetails id={termsTemplateId} />
    </main>
  )
}

export default TermsTemplate
