#!/usr/bin/env bash

# Converts all Markdown manuals into PDFs via wkhtmltopdf with Pandoc
# Usage: cd cypress/manuals && ./local-pdf.sh

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
MANUALS_SRC="${REPO_ROOT}/apps/admin-panel/cypress/manuals"
MANUALS_DIR="${MANUALS_SRC}/results"
mkdir -p "$MANUALS_DIR"

# Generate temporary CSS
cat > temp.css << 'EOL'
body {
    width: 100%;
    margin: 0;
    padding: 5mm;
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
    line-height: 1.4;
    font-size: 16px;
    word-wrap: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
}
img {
    display: block;
    width: 100%;
    max-height: 80vh;
    object-fit: contain;
    margin: 15px auto;
}
.page {
    width: 100%;
    max-width: none;
    margin: 0 auto;
    box-sizing: border-box;
}
figure {
    margin: 0;
    padding: 0;
    text-align: center;
}
figcaption {
    text-align: center;
    margin: 10px 0;
}
p {
    margin: 0.5em 0;
    max-width: 100%;
    box-sizing: border-box;
    word-break: normal;
    overflow-wrap: anywhere;
}
p:has(img) {
    margin: 0;
    padding: 0;
    text-align: center;
}
p img + em {
    display: block;
    text-align: center;
    margin-top: 5px;
}
h1, h2, h3, h4, h5, h6 {
    max-width: 100%;
    box-sizing: border-box;
    word-break: normal;
    overflow-wrap: anywhere;
}
* {
    max-width: 100%;
    box-sizing: border-box;
}

hr {
    border: none;
    border-bottom: 1px solid;
    margin: 1.5em 0;
}

img {
    page-break-inside: avoid;
}

.pb {
    page-break-before: always !important;
    margin: 0 !important;
    padding: 0 !important;
}

a {
    color: #0066cc;
    text-decoration: underline;
}
EOL

# Process each Markdown file
for file in *.md; do
    # Determine output filename
    CUSTOM_FILENAME=$(grep -m 1 "^pdf_filename:" "$file" | sed 's/pdf_filename: *//' | tr -d '[:space:]')
    if [ -z "$CUSTOM_FILENAME" ]; then
        OUTPUT_FILENAME="${file%.md}"
    else
        OUTPUT_FILENAME="$CUSTOM_FILENAME"
    fi

    # Preprocess page breaks and strip pdf_filename
    sed 's/<!-- new-page -->/<div class="pb"><\/div>/g' "$file" > temp.md
    sed -i '/^pdf_filename:/d' temp.md

    # Convert to PDF
    pandoc temp.md \
      --standalone --embed-resources \
      --metadata title="${OUTPUT_FILENAME}" -V title="" \
      -o "${MANUALS_DIR}/${OUTPUT_FILENAME}.pdf" \
      --pdf-engine=weasyprint \
      --css temp.css \
      -V papersize=a4 \
      -V margin-top=10mm \
      -V margin-right=5mm \
      -V margin-bottom=5mm \
      -V margin-left=5mm

    echo "Converted $file → results/${OUTPUT_FILENAME}.pdf"
    rm temp.md
done

# Clean up
rm temp.css

echo "Manuals generated in ${MANUALS_DIR}"
