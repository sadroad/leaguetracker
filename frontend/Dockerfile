FROM node:20-bookworm as base
WORKDIR /app
RUN npm i -g pnpm

FROM base as dependencies
COPY pnpm-lock.yaml ./
RUN pnpm fetch

FROM base as build
ADD . .
COPY --from=dependencies /app/node_modules ./node_modules
RUN pnpm install -r --offline
RUN pnpm build

FROM node:20-alpine
COPY --from=build /app/dist ./dist
COPY --from=build /app/server ./server
COPY --from=build /app/node_modules ./node_modules/

CMD ["node", "server/entry.express"]
