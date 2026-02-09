"use client"

import React, { useEffect, useState, useRef } from "react"

import { Button } from "@lana/web/ui/button"

import { Skeleton } from "@lana/web/ui/skeleton"

interface D2RendererProps {
  d2Source: string
}

const D2Renderer: React.FC<D2RendererProps> = ({ d2Source }) => {
  const [svg, setSvg] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    let cancelled = false

    const renderD2 = async () => {
      setLoading(true)
      setError(null)
      try {
        const { D2 } = await import("@terrastruct/d2")
        const d2 = new D2()
        const result = await d2.compile(d2Source)
        const renderedSvg = await d2.render(result.diagram, result.renderOptions)
        if (!cancelled) {
          setSvg(renderedSvg)
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to render D2 diagram")
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    renderD2()
    return () => {
      cancelled = true
    }
  }, [d2Source])

  const handleDownload = () => {
    const blob = new Blob([d2Source], { type: "text/plain" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = "account-hierarchy.d2"
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <Button variant="outline" size="sm" onClick={handleDownload}>
          Download D2
        </Button>
      </div>

      {loading && (
        <div className="space-y-2">
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-64 w-full" />
        </div>
      )}

      {error && (
        <div className="space-y-2">
          <p className="text-destructive text-sm">{error}</p>
          <details>
            <summary className="cursor-pointer text-sm text-muted-foreground">
              Raw D2 Source
            </summary>
            <pre className="mt-2 rounded bg-muted p-4 text-xs overflow-auto max-h-96">
              {d2Source}
            </pre>
          </details>
        </div>
      )}

      {svg && !loading && (
        <div
          ref={containerRef}
          className="overflow-auto border rounded-md p-4 bg-white"
          dangerouslySetInnerHTML={{ __html: svg }}
        />
      )}
    </div>
  )
}

export default D2Renderer
