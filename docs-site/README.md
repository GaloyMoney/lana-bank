## Docs Site

### Dev server

Install dependencies first:

```bash
pnpm install
```

Run the English locale (default):

```bash
pnpm run start -- --locale en
```

Run the Spanish locale:

```bash
pnpm run start -- --locale es
```

If you need a specific port, add `--port 3001` (or any free port):

```bash
pnpm run start -- --locale en --port 3001
```

### Production build and serve

Build the production site (both locales):

```bash
pnpm run build
```

Serve the production build locally:

```bash
pnpm run serve
```

### Search

The site uses local search (`@easyops-cn/docusaurus-search-local`) which supports both English and Spanish.

**Note:** Search is only available after running `pnpm run build`. It does not work in dev mode (`pnpm run start`). To test search functionality, use:

```bash
pnpm run build && pnpm run serve
```
