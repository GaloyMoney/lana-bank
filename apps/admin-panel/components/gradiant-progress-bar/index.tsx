export const GradientProgressBar = ({ percentage }: { percentage: number }) => {
  return (
    <div className="relative w-[300px] mt-[20px]">
      <div className="w-full h-[3px] bg-gradient-to-r from-green-500 via-yellow-500 to-red-500 relative">
        <div
          style={{
            position: "absolute",
            top: "-32px",
            left: `calc(${percentage}%)`,
            transform: "translateX(-50%)",
            whiteSpace: "nowrap",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
          }}
        >
          <div className="text-xs mb-[2px]">{percentage}%</div>
          <div className="text-xl leading-[10px]">▼</div>
        </div>
      </div>
      <div className="flex justify-between w-full mt-[5px]">
        <div>0%</div>
        <div className="text-center ml-[2ch]">50%</div>
        <div className="text-right">100%</div>
      </div>
    </div>
  )
}
