// components/logo/index.tsx
import Image from "next/image"
import logoNeutral from "./logo-neutral.svg"
import logoPrimary from "./logo-primary.svg"

type LogoProps = {
  variant?: "neutral" | "primary"
  className?: string
}

const Logo: React.FC<LogoProps> = ({ variant = "neutral", className }) => {
  const logoSrc = variant === "primary" ? logoPrimary : logoNeutral

  return <Image className={className} src={logoSrc} alt={`${variant} logo`} priority />
}

export default Logo
