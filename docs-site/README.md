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

