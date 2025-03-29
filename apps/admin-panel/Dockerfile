# Base image
FROM node:20-alpine AS base

FROM base AS builder
RUN apk add --no-cache libc6-compat
WORKDIR /app

RUN npm install -g pnpm@9.15.2

COPY pnpm-workspace.yaml pnpm-lock.yaml ./

COPY apps/admin-panel/package.json ./apps/admin-panel/
COPY apps/shared-web/package.json ./apps/shared-web/

COPY apps/shared-web ./apps/shared-web
COPY apps/admin-panel ./apps/admin-panel

RUN pnpm install --frozen-lockfile --filter admin-panel...

WORKDIR /app/apps/admin-panel
RUN pnpm run build

FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/apps/admin-panel/public ./public

COPY --from=builder --chown=nextjs:nodejs \
  /app/apps/admin-panel/.next/standalone ./

COPY --from=builder --chown=nextjs:nodejs \
  /app/apps/admin-panel/.next/static \
  ./apps/admin-panel/.next/static

USER nextjs

EXPOSE 3000

ENV PORT=3000

CMD HOSTNAME="0.0.0.0" node apps/admin-panel/server.js