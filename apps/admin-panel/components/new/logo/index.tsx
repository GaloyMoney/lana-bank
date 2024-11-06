"use client"

import LogoNeutral from "./logo-neutral.svg"
import LogoPrimary from "./logo-primary.svg"

type LogoProps = {
  variant?: "neutral" | "primary"
  width?: number
  className?: string
}

const Logo: React.FC<LogoProps> = ({ variant = "neutral", className, width }) => {
  const LogoSrc = variant === "primary" ? LogoPrimary : LogoNeutral

  return <LogoSrc className={className} width={width} />
}

export default Logo
