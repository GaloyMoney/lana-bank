"use client"

import Chart from "react-apexcharts"

type CollateralUsdChartProps = {
  data: {
    value: number
    date: Date
  }[]
}

const CollateralUsdChart: React.FC<CollateralUsdChartProps> = ({ data }) => {
  const seriesData = data.map((point) => ({
    x: point.date.getTime(), // Convert date to timestamp
    y: point.value / 100, // Convert value to USD
  }))

  // Determine the maximum value for the y-axis with a 10% buffer
  const maxYValue = Math.max(...seriesData.map((point) => point.y)) * 1.1

  return (
    <div className="w-full">
      <Chart
        type="area"
        height={250}
        series={[
          {
            name: "Collateral Value (USD)",
            data: seriesData,
          },
        ]}
        options={{
          chart: {
            toolbar: {
              show: false,
            },
          },
          dataLabels: {
            enabled: false,
          },
          xaxis: {
            type: "datetime",
            axisTicks: {
              show: false,
            },
            axisBorder: {
              show: false,
            },
            labels: {
              format: "MMM dd", // Format x-axis labels as month-day (e.g., "Apr 05")
              style: {
                colors: "#757575",
                fontSize: "12px",
                fontFamily: "inherit",
                fontWeight: 300,
              },
            },
          },
          yaxis: {
            labels: {
              formatter: (value: number) => `$${value.toLocaleString()}`, // Format y-axis labels as USD
              style: {
                colors: "#757575",
                fontSize: "12px",
                fontFamily: "inherit",
                fontWeight: 300,
              },
            },
            max: maxYValue, // Set max y-axis value to extend the grid
          },
          grid: {
            show: true,
            borderColor: "#EEEEEE",
            strokeDashArray: 5,
            xaxis: {
              lines: {
                show: true,
              },
            },
            padding: {
              top: 5,
              right: 20,
            },
          },
          tooltip: {
            theme: "light",
            x: {
              format: "MMM dd, yyyy", // Format tooltip date as "Month day, year"
            },
            y: {
              formatter: (value: number) => `$${value.toLocaleString()}`, // Format tooltip value as USD
            },
          },
          colors: ["#2196F3"],
          stroke: {
            lineCap: "round",
            width: 2,
          },
          fill: {
            type: "gradient",
            gradient: {
              shadeIntensity: 1,
              inverseColors: false,
              opacityFrom: 0.4,
              opacityTo: 0.05,
              stops: [20, 100, 100, 100],
            },
            opacity: 0.8,
          },
        }}
      />
    </div>
  )
}

export default CollateralUsdChart
