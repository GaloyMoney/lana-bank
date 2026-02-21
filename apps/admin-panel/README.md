This is a [Next.js](https://nextjs.org/) project

## Getting Started

First, run the development server:

```bash
pnpm dev
```

### Steps to log in to the admin panel locally:

1. Go to `http://admin.localhost:4455` URL to open the admin panel login.
2. Enter the email. For now, we have two allowed emails: `admin@galoy.io`.
3. Get the login code by running `make get-superadmin-login-code` or `make get-admin-login-code EMAIL=admin@galoy.io` in the project root.
4. Enter the code to log in to the admin panel.

### Testing with Cypress

1. Ensure `cypress` binary is available (or install with `$ pnpm install cypress`)
2. Add following entries to `/etc/hosts`
  ```
  127.0.0.1   admin.localhost
  ::1   admin.localhost
  ```
3. Execute with `pnpm cypress:run-headless`
