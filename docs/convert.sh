# cat .claude/commands/mermaid.md | pnpm claude --dangerously-skip-permissions -p
# cp ../../slidev/pages/*.md slidev/

pnpm mmdc -i core-banking-manual.md -o core-banking-manual-svg.md
pnpm md-to-pdf core-banking-manual-svg.md
