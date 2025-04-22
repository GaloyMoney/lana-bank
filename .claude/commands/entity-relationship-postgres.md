create a entity relationship mermaid diagrams for each sql files, including tables properties located in lana/app/migrations/
output should be saved in docs/er/ under lana.md and cala.md
make sure the mermaid compile by running `npx -p @mermaid-js/mermaid-cli mmdc -i docs/er/{lana|cala}.md -o test.md`
when complete, verify that every entity in any .sql migration file appears in the mermaid file
also, you can remove test.md, we just want to use `mmdc` to verify the code generated compiles
