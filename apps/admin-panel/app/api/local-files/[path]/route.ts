import { NextResponse } from "next/server"
import { readFile } from "fs/promises"

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ path: string }> },
) {
  // Only allow in development mode for security
  if (process.env.NODE_ENV !== "development") {
    return NextResponse.json(
      { error: "This endpoint is only available in development mode" },
      { status: 403 },
    )
  }

  try {
    const { path } = await params

    // Decode the base64-encoded file path
    const decodedPath = Buffer.from(decodeURIComponent(path), "base64").toString("utf-8")

    // Read the file
    const fileBuffer = await readFile(decodedPath)

    // Determine the MIME type based on file extension
    const extension = decodedPath.split(".").pop()?.toLowerCase()
    const mimeType = getMimeType(extension)

    // Return the file with appropriate headers
    return new NextResponse(fileBuffer, {
      headers: {
        "Content-Type": mimeType,
        "Content-Disposition": `inline; filename="${decodedPath.split("/").pop()}"`,
      },
    })
  } catch (error) {
    console.error("Error serving local file:", error)
    return NextResponse.json({ error: "File not found" }, { status: 404 })
  }
}

function getMimeType(extension: string | undefined): string {
  const mimeTypes: Record<string, string> = {
    pdf: "application/pdf",
    csv: "text/csv",
    txt: "text/plain",
  }

  return extension ? mimeTypes[extension] || "application/octet-stream" : "application/octet-stream"
}
