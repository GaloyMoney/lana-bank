## Docs Site

### Dev server

Install dependencies first:

```bash
npm install
```

Run the English locale (default):

```bash
npm run start -- --locale en
```

Run the Spanish locale:

```bash
npm run start -- --locale es
```

If you need a specific port, add `--port 3001` (or any free port):

```bash
npm run start -- --locale en --port 3001
```

### Production build and serve

Build the production site (both locales):

```bash
npm run build
```

Serve the production build locally:

```bash
npm run serve
```

### Search

The site uses local search (`@easyops-cn/docusaurus-search-local`) which supports both English and Spanish.

**Note:** Search is only available after running `npm run build`. It does not work in dev mode (`npm run start`). To test search functionality, use:

```bash
npm run build && npm run serve
```

