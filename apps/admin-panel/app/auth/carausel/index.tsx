"use client"

import MTCarousel from "@material-tailwind/react/components/Carousel"

import GenerateIcon from "./vectors/generate.svg"
import ManageIcon from "./vectors/manage.svg"
import OneStopIcon from "./vectors/onestop.svg"

const Carousel: React.FC = () => {
  return (
    <div className="px-10 flex h-full">
      <MTCarousel
        loop
        autoplay
        prevArrow={() => <></>}
        nextArrow={() => <></>}
        navigation={({ setActiveIndex, activeIndex, length }) => (
          <div className="absolute bottom-20 left-2/4 z-50 flex -translate-x-2/4 gap-2">
            {new Array(length).fill("").map((_, i) => (
              <span
                key={i}
                className={`block h-1 cursor-pointer rounded-2xl transition-all content-[''] ${
                  activeIndex === i ? "w-8 bg-white" : "w-4 bg-white/50"
                }`}
                onClick={() => setActiveIndex(i)}
              />
            ))}
          </div>
        )}
      >
        <CarouselItem
          text="One-stop view into the bank's financials"
          Icon={OneStopIcon}
        />
        <CarouselItem
          text="Manage customers, approve loans, record deposits and withdrawals"
          Icon={ManageIcon}
        />
        <CarouselItem
          text="Generate regulatory reporting for government compliance"
          Icon={GenerateIcon}
        />
      </MTCarousel>
    </div>
  )
}

export default Carousel

type CarouselItemProps = {
  text: string
  Icon: React.FunctionComponent<React.SVGProps<SVGSVGElement>>
}

const CarouselItem: React.FC<CarouselItemProps> = ({ text, Icon }) => (
  <div className="flex flex-col justify-center items-center space-y-10 h-full">
    <Icon className="w-[300px] h-[300px]" />
    <div className="text-title text-body-lg text-center !text-white">{text}</div>
  </div>
)
