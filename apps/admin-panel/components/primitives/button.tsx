"use client"

import { ButtonHTMLAttributes } from "react"

import { Button as MTButton } from "@material-tailwind/react"

type ButtonProps = {
  title: string
  type?: ButtonHTMLAttributes<HTMLButtonElement>["type"]
  onClick?: () => void
  className?: string
  size?: React.ComponentProps<typeof MTButton>["size"]
}
const Button: React.FC<ButtonProps> = ({
  title,
  type = "button",
  className = "",
  // eslint-disable-next-line no-empty-function
  onClick = () => {},
  size,
}) => {
  return (
    <MTButton
      className={`bg-action-secondary ${className}`}
      type={type}
      onClick={onClick}
      size={size}
    >
      {title}
    </MTButton>
  )
}

export default Button
