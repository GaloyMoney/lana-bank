"use client"

import { ButtonHTMLAttributes } from "react"

import { Button as MTButton } from "@material-tailwind/react"

type ButtonProps = {
  title: string
  type?: ButtonHTMLAttributes<HTMLButtonElement>["type"]
  onClick?: () => void
  className?: string
  size?: React.ComponentProps<typeof MTButton>["size"]
  icon?: React.ReactNode
}

const Button: React.FC<ButtonProps> = ({
  title,
  icon,
  type = "button",
  className = "",
  // eslint-disable-next-line no-empty-function
  onClick = () => {},
  size,
}) => {
  return (
    <MTButton
      className={`bg-action-secondary flex justify-center items-center gap-2 ${className}`}
      type={type}
      onClick={onClick}
      size={size}
      suppressHydrationWarning
    >
      <span>{icon}</span>
      <span>{title}</span>
    </MTButton>
  )
}

export default Button
