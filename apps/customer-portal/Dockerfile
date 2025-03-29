# Base image
FROM node:20-alpine AS base

FROM base AS builder
RUN apk add --no-cache libc6-compat
WORKDIR /src

RUN npm install -g pnpm@9.15.2

COPY pnpm-workspace.yaml pnpm-lock.yaml ./

COPY apps/customer-portal/package.json ./apps/customer-portal/
COPY apps/shared-web/package.json ./apps/shared-web/

COPY apps/shared-web ./apps/shared-web
COPY apps/customer-portal ./apps/customer-portal

RUN pnpm install --frozen-lockfile --filter customer-portal...

WORKDIR /src/apps/customer-portal
RUN pnpm run build

FROM base AS runner
WORKDIR /src

ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /src/apps/customer-portal/public ./public

COPY --from=builder --chown=nextjs:nodejs \
  /src/apps/customer-portal/.next/standalone ./

COPY --from=builder --chown=nextjs:nodejs \
  /src/apps/customer-portal/.next/static \
  ./apps/customer-portal/.next/static

USER nextjs

EXPOSE 3000

ENV PORT=3000

CMD HOSTNAME="0.0.0.0" node apps/customer-portal/server.js