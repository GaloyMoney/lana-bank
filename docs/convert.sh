# cat .claude/commands/mermaid.md | pnpm claude --dangerously-skip-permissions -p
# cp ../../slidev/pages/*.md slidev/

# Loop through all .md files in the output directory, excluding *-svg.md files
# The script runs in docs/, so these paths are relative to docs/
for infile in output/*.md; do
  # Skip files that already have -svg in their name
  if [[ "$infile" == *"-svg.md" ]]; then
    continue
  fi

  # Construct filenames relative to the docs/ directory
  base=$(basename "$infile" .md)
  outfile="output/${base}-svg.md"

  # Construct paths relative to the workspace root for pnpm commands
  infile_from_root="docs/$infile"
  outfile_from_root="docs/$outfile"

  echo "Processing $infile_from_root -> $outfile_from_root (relative to workspace root)"
  # Use paths relative to workspace root for pnpm mmdc
  pnpm mmdc -i "$infile_from_root" -o "$outfile_from_root"

  # The check below uses paths relative to docs/ where the script runs
  echo "Checking for $outfile and converting to PDF"
  if [ -f "$outfile" ]; then
    # Use path relative to workspace root for pnpm md-to-pdf
    pnpm md-to-pdf "$outfile_from_root"
  else
    # This echo uses the path relative to docs/
    echo "Warning: SVG file $outfile not found, skipping PDF conversion for $infile"
  fi
done
