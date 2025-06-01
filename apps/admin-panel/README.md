This is a [Next.js](https://nextjs.org/) project

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
```

### Steps to log in to the admin panel locally:

1. Go to `http://localhost:4455/admin` URL to open the admin panel login.
2. Enter the email. For now, we have two allowed emails: `admin@galoy.io`.
3. Get the login code by running `make get-superadmin-login-code` or `make get-admin-login-code EMAIL=admin@galoy.io` in the project root.
4. Enter the code to log in to the admin panel.
