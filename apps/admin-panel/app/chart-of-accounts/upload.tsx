"use client"

import React, { useCallback, useState } from "react"
import { useDropzone } from "react-dropzone"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { FiUpload, FiFile, FiAlertTriangle } from "react-icons/fi"

import { Button } from "@lana/web/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@lana/web/ui/alert"
import { Card, CardContent } from "@lana/web/ui/card"
import { toast } from "sonner"

import {
  ChartOfAccountsDocument,
  useChartOfAccountsCsvImportMutation,
} from "@/lib/graphql/generated"

gql`
  mutation ChartOfAccountsCsvImport($input: ChartOfAccountsCsvImportInput!) {
    chartOfAccountsCsvImport(input: $input) {
      success
    }
  }
`

const ChartOfAccountsUpload = ({ chartId }: { chartId: string }) => {
  const t = useTranslations("ChartOfAccounts.upload")
  const [file, setFile] = useState<File | null>(null)
  const [uploading, setUploading] = useState(false)
  const [uploadError, setUploadError] = useState<string | null>(null)

  const [uploadCsv] = useChartOfAccountsCsvImportMutation({
    refetchQueries: [ChartOfAccountsDocument],
  })

  const onDrop = useCallback((acceptedFiles: File[]) => {
    if (acceptedFiles.length > 0) {
      setFile(acceptedFiles[0])
      setUploadError(null)
    }
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      "text/csv": [".csv"],
    },
    maxFiles: 1,
  })

  const handleUpload = async () => {
    if (!file) {
      setUploadError(t("noFileSelected"))
      return
    }
    setUploading(true)
    setUploadError(null)
    try {
      const result = await uploadCsv({
        variables: {
          input: {
            chartId: chartId,
            file: file,
          },
        },
      })
      if (result.data?.chartOfAccountsCsvImport.success) {
        toast.success(t("successDescription"))
        resetUpload()
      } else {
        setUploadError(t("genericError"))
      }
    } catch (error) {
      setUploadError(error instanceof Error ? error.message : t("genericError"))
    } finally {
      setUploading(false)
    }
  }

  const resetUpload = () => {
    setFile(null)
    setUploadError(null)
  }

  return (
    <Card className="border-none shadow-none">
      <CardContent>
        <Card
          {...getRootProps()}
          className={`
            border-2 border-dashed rounded-md p-8 transition-colors cursor-pointer
            ${isDragActive ? "border-primary bg-primary/5" : "border-muted-foreground/25"}
            ${uploading ? "pointer-events-none opacity-70" : ""}
          `}
        >
          <input {...getInputProps()} />
          <CardContent className="flex flex-col items-center justify-center gap-3 text-center p-4">
            {file ? (
              <>
                <div className="w-14 h-14 rounded-full bg-primary/10 flex items-center justify-center text-primary">
                  <FiFile className="w-7 h-7" />
                </div>
                <p className="text-sm font-medium text-foreground">{file.name}</p>
              </>
            ) : (
              <>
                <div className="w-14 h-14 rounded-full bg-muted flex items-center justify-center text-muted-foreground">
                  <FiUpload className="w-7 h-7" />
                </div>
                <p className="text-sm font-medium">
                  {isDragActive ? t("dropHere") : t("dragAndDrop")}
                </p>
                <p className="text-xs text-muted-foreground">{t("csvOnly")}</p>
              </>
            )}
          </CardContent>
        </Card>

        {uploading && (
          <Card className="border-none shadow-none">
            <CardContent className="p-2">
              <p className="text-sm text-muted-foreground text-center">
                {t("uploading")}
              </p>
            </CardContent>
          </Card>
        )}

        {uploadError && (
          <Alert variant="destructive">
            <FiAlertTriangle className="h-4 w-4" />
            <AlertTitle>{t("errorTitle")}</AlertTitle>
            <AlertDescription>{uploadError}</AlertDescription>
          </Alert>
        )}

        <div className="flex justify-end gap-2 mt-4">
          {file && (
            <Button onClick={resetUpload} variant="outline" disabled={uploading}>
              {t("reset")}
            </Button>
          )}
          <Button onClick={handleUpload} disabled={!file || uploading} className="gap-2">
            <FiUpload className="h-4 w-4" />
            {t("upload")}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default ChartOfAccountsUpload
