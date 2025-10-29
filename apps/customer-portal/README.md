This is a [Next.js](https://nextjs.org/) project

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

### Testing with Cypress

1. Ensure `cypress` binary is available (or install with `$ pnpm install cypress`)
2. Add following entries to `/etc/hosts`
  ```
  127.0.0.1   app.localhost
  ::1   app.localhost
  ```
3. Execute with `pnpm cypress:run-headless`
