#!/usr/bin/env bash

REPO_ROOT=$(git rev-parse --show-toplevel)
MANUALS_DIR="${REPO_ROOT}/apps/admin-panel/cypress/manuals/results"
mkdir -p "$MANUALS_DIR"

for file in *.md; do
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

    # First, preprocess the markdown to convert special page break markers to HTML
    sed 's/<!-- new-page -->/<div class="pb"><\/div>/g' "$file" > "temp.md"

    pandoc --metadata title="${file%.md}" -V title="" "temp.md" \
        -o "${MANUALS_DIR}/${file%.md}.pdf" \
        --pdf-engine=wkhtmltopdf \
        --pdf-engine-opt=--enable-local-file-access \
        --pdf-engine-opt=--enable-internal-links \
        --pdf-engine-opt=--print-media-type \
        --pdf-engine-opt=--no-stop-slow-scripts \
        -V papersize=a4 \
        --css temp.css \
        -V margin-top=10mm -V margin-right=5mm -V margin-bottom=5mm -V margin-left=5mm
    echo "Converted $file to results/${file%.md}.pdf"
    rm temp.md
done
rm temp.css

echo "Manuals generated in ${MANUALS_DIR}"
