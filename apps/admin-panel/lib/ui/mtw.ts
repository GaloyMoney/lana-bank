"use client"

export * from "@material-tailwind/react"

type EventCapture = {
  onPointerEnterCapture?: unknown
  onPointerLeaveCapture?: unknown
}

declare module "@material-tailwind/react" {
  interface CarouselProps extends EventCapture {
    placeholder?: undefined
  }
  interface ButtonProps extends EventCapture {
    placeholder?: undefined
  }
}
