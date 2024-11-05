/* eslint-disable @typescript-eslint/no-unused-vars */

import { CarouselProps } from "@material-tailwind/react/components/Carousel"
import { ButtonProps } from "@material-tailwind/react"

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
