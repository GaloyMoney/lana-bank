import classNames from "classnames"

type PillColor =
  | "teal"
  | "brown"
  | "orange"
  | "cyan"
  | "light-green"
  | "purple"
  | "lime"
  | "blue-gray"

type PillProps = {
  color: PillColor
  className?: string
  border?: boolean
}

const Pill: React.FC<React.PropsWithChildren<PillProps>> = ({
  children,
  className = "",
  color,
  border = false,
}) => {
  const { bg: bgClass, text: textClass, border: borderClass } = pillColors[color]

  const pillClasses = classNames(
    "p-1 rounded-sm text-title-xs !font-bold uppercase",
    bgClass,
    textClass,
    className,
    {
      border: border,
      [borderClass]: border,
    },
  )

  return <div className={pillClasses}>{children}</div>
}

export default Pill

// prettier-ignore
const pillColors: Record<PillColor, { bg: string; text: string; border: string }> = {
  "teal": { bg: "!bg-teal-50", text: "!text-teal-500", border: "!border-teal-500" },
  "brown": { bg: "!bg-brown-50", text: "!text-brown-500", border: "!border-brown-500" },
  "orange": { bg: "!bg-orange-50", text: "!text-orange-500", border: "!border-orange-500" },
  "cyan": { bg: "!bg-cyan-50", text: "!text-cyan-500", border: "!border-cyan-500" },
  "light-green": { bg: "!bg-light-green-50", text: "!text-light-green-500", border: "!border-light-green-500" },
  "purple": { bg: "!bg-purple-50", text: "!text-purple-500", border: "!border-purple-500" },
  "lime": { bg: "!bg-lime-50", text: "!text-lime-800", border: "!border-lime-800" },
  "blue-gray": { bg: "!bg-blue-gray-50", text: "!text-blue-gray-500", border: "!border-blue-gray-500" },
}
