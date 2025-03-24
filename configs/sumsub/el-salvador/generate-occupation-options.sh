#!/bin/sh
jq -R -s -c '
	split("\n") | .[1:] | map(select(length > 0)) |
	map(split(",")) |
	map({
		value: (.[0] | tonumber | tostring | ("000" + .)[-3:]),
		title: .[2],
		localizedTitle: {
			values: [
				{lang: "en", value: .[2]},
				{lang: "es", value: .[1]}
			]
		},
		score: null
	})
' list-of-occupations.csv
