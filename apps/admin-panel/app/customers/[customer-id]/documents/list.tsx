"use client"

import React, { useCallback, useEffect, useState } from "react"
import { useDropzone } from "react-dropzone"
import { gql, ApolloError } from "@apollo/client"
import { CgSpinner } from "react-icons/cg"
import { toast } from "sonner"
import { useTranslations } from "next-intl"

import { Card, CardContent } from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"

import DataTable, { Column } from "@/components/data-table"

import {
  GetCustomerDocumentsDocument,
  GetCustomerDocumentsQuery,
  useCustomerDocumentAttachMutation,
  useCustomerDocumentDeleteMutation,
  useCustomerDocumentDownloadLinkGenerateMutation,
} from "@/lib/graphql/generated"
import CardWrapper from "@/components/card-wrapper"

type DocumentType = NonNullable<
  GetCustomerDocumentsQuery["customerByPublicId"]
>["documents"][number]
type DocumentProps = {
  customer: NonNullable<GetCustomerDocumentsQuery["customerByPublicId"]>
  refetch: () => void
}

export const Documents: React.FC<DocumentProps> = ({ customer, refetch }) => {
  return (
    <>
      <CustomerDocuments documents={customer.documents} refetch={refetch} />
      <AddDocument customer={customer} refetch={refetch} />
    </>
  )
}

gql`
  mutation CustomerDocumentDownloadLinkGenerate(
    $input: CustomerDocumentDownloadLinksGenerateInput!
  ) {
    customerDocumentDownloadLinkGenerate(input: $input) {
      link
    }
  }

  mutation CustomerDocumentDelete($input: CustomerDocumentDeleteInput!) {
    customerDocumentDelete(input: $input) {
      deletedDocumentId
    }
  }

  mutation CustomerDocumentAttach($file: Upload!, $customerId: UUID!) {
    customerDocumentAttach(input: { file: $file, customerId: $customerId }) {
      document {
        id
        customerId
        filename
      }
    }
  }
`

type CustomerDocumentsProps = {
  documents: NonNullable<GetCustomerDocumentsQuery["customerByPublicId"]>["documents"]
  refetch: () => void
}

const CustomerDocuments: React.FC<CustomerDocumentsProps> = ({ documents, refetch }) => {
  const t = useTranslations("Customers.CustomerDetails.Documents")

  const [linkLoading, setLinkLoading] = useState<{ [key: string]: boolean }>({})
  const [deleteLoading, setDeleteLoading] = useState<{ [key: string]: boolean }>({})

  const [customerDocumentDownloadLinkGenerate] =
    useCustomerDocumentDownloadLinkGenerateMutation()
  const [customerDocumentDelete] = useCustomerDocumentDeleteMutation({
    refetchQueries: [GetCustomerDocumentsDocument],
  })

  const openFile = useCallback(
    async (id: string) => {
      setLinkLoading((prev) => ({ ...prev, [id]: true }))

      const { data } = await customerDocumentDownloadLinkGenerate({
        variables: { input: { documentId: id } },
      }).finally(() => setLinkLoading((prev) => ({ ...prev, [id]: false })))

      if (!data?.customerDocumentDownloadLinkGenerate?.link) {
        toast.error(t("messages.generateLinkError"))
        return
      }

      window.open(data.customerDocumentDownloadLinkGenerate.link, "_blank")
    },
    [customerDocumentDownloadLinkGenerate, t],
  )

  const deleteDocument = useCallback(
    async (id: string) => {
      setDeleteLoading((prev) => ({ ...prev, [id]: true }))

      try {
        await customerDocumentDelete({
          variables: { input: { documentId: id } },
        })
        toast.success(t("messages.deleteSuccess"))
        refetch()
      } catch (error) {
        toast.error(t("messages.deleteError"))
        console.error("Delete document error:", error)
      } finally {
        setDeleteLoading((prev) => ({ ...prev, [id]: false }))
      }
    },
    [customerDocumentDelete, refetch, t],
  )

  const columns: Column<DocumentType>[] = [
    {
      key: "documentId",
      header: t("columns.id"),
    },
    {
      key: "filename",
      header: t("columns.fileName"),
    },
    {
      key: "documentId",
      header: "",
      render: (_, document) => (
        <div className="flex justify-end space-x-2">
          <Button
            variant="outline"
            onClick={(e) => {
              e.stopPropagation()
              openFile(document.documentId)
            }}
            disabled={linkLoading[document.documentId]}
          >
            {linkLoading[document.documentId] ? (
              <CgSpinner className="animate-spin h-5 w-5" />
            ) : (
              t("buttons.view")
            )}
          </Button>
          <Button
            variant="ghost"
            onClick={(e) => {
              e.stopPropagation()
              deleteDocument(document.documentId)
            }}
            disabled={deleteLoading[document.documentId]}
          >
            {deleteLoading[document.documentId] ? (
              <CgSpinner className="animate-spin h-5 w-5" />
            ) : (
              t("buttons.delete")
            )}
          </Button>
        </div>
      ),
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable data={documents} columns={columns} />
    </CardWrapper>
  )
}

const AddDocument: React.FC<DocumentProps> = ({ customer, refetch }) => {
  const t = useTranslations("Customers.CustomerDetails.Documents")

  const [customerDocumentAttach, { loading }] = useCustomerDocumentAttachMutation({
    refetchQueries: [GetCustomerDocumentsDocument],
  })

  const handleFileUpload = useCallback(
    async (file: File) => {
      try {
        await customerDocumentAttach({
          variables: {
            customerId: customer.customerId,
            file,
          },
        })
        refetch()
        toast.success(t("messages.uploadSuccess"))
      } catch (err) {
        const errorMessage = getErrorMessage(err, t)
        toast.error(errorMessage)
      }
    },
    [customerDocumentAttach, customer.customerId, refetch, t],
  )

  const onDrop = useCallback(
    async (acceptedFiles: File[]) => {
      if (acceptedFiles.length > 0) {
        const file = acceptedFiles[0]
        if (file.type === "application/pdf") {
          await handleFileUpload(file)
        } else {
          toast.error(t("messages.pdfOnly"))
        }
      }
    },
    [handleFileUpload, t],
  )

  const { getRootProps, getInputProps } = useDropzone({
    onDrop,
    multiple: false,
  })

  const handlePaste = useCallback(
    (event: ClipboardEvent) => {
      const items = event.clipboardData?.items
      if (items) {
        if (
          items.length === 1 &&
          items[0].kind === "file" &&
          items[0].type === "application/pdf"
        ) {
          const file = items[0].getAsFile()
          if (file) {
            handleFileUpload(file)
          } else {
            toast.error(t("messages.pasteError"))
          }
        } else {
          toast.error(t("messages.pdfOnly"))
        }
      }
    },
    [handleFileUpload, t],
  )

  useEffect(() => {
    document.addEventListener("paste", handlePaste)
    return () => document.removeEventListener("paste", handlePaste)
  }, [handlePaste])

  return (
    <div {...getRootProps()}>
      <input {...getInputProps()} id="fileInput" disabled={loading} />
      <Card className="mt-2 cursor-pointer">
        <CardContent className="p-6 flex justify-center items-center w-full h-20">
          {loading ? (
            <CgSpinner className="animate-spin h-5 w-5" />
          ) : (
            <span>{t("dropzone.prompt")}</span>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

const getErrorMessage = (
  err: unknown,
  t: ReturnType<typeof useTranslations<"Customers.CustomerDetails.Documents">>,
): string => {
  if (
    err instanceof ApolloError &&
    err.networkError &&
    "statusCode" in err.networkError
  ) {
    if (err.networkError.statusCode === 413) {
      return t("messages.fileTooLarge")
    }
    return t("messages.uploadFailed", { message: err.message })
  }
  if (err instanceof Error) {
    return t("messages.uploadFailed", { message: err.message })
  }
  return t("messages.unexpectedError")
}

export default Documents
