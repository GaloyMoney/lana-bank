# npx -p @anthropic-ai/claude-code 
# cat .claude/commands/mermaid.md | claude --dangerously-skip-permissions -p

# cp ../../slidev/pages/*.md slidev/

npx -p @mermaid-js/mermaid-cli mmdc -i core-banking-manual.md -o core-banking-manual-svg.md
npx -p md-to-pdf md-to-pdf core-banking-manual-svg.md
