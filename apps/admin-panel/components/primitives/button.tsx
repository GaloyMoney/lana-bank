"use client"

import { ButtonHTMLAttributes } from "react"

import { Button as MTButton } from "@/lib/ui/mtw"

type ButtonProps = {
  title: string
  type?: ButtonHTMLAttributes<HTMLButtonElement>["type"]
  onClick?: () => void
  className?: string
  size?: React.ComponentProps<typeof MTButton>["size"]
  icon?: React.ReactNode
  rightIcon?: React.ReactNode
  variant?: React.ComponentProps<typeof MTButton>["variant"]
  loading?: boolean
}

const Button: React.FC<ButtonProps> = ({
  title,
  icon,
  rightIcon,
  type = "button",
  className = "",
  // eslint-disable-next-line no-empty-function
  onClick = () => {},
  size,
  variant,
  loading,
}) => {
  return (
    <MTButton
      className={`flex justify-center items-center gap-2 ${className}`}
      type={type}
      onClick={onClick}
      size={size}
      suppressHydrationWarning
      variant={variant}
      loading={loading}
    >
      <span>{icon}</span>
      <span>{title}</span>
      <span>{rightIcon}</span>
    </MTButton>
  )
}

export default Button
