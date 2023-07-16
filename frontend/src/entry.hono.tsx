import {
  createQwikCity,
  type PlatformNode,
} from "@builder.io/qwik-city/middleware/node";
import qwikCityPlan from "@qwik-city-plan";
import { manifest } from "@qwik-client-manifest";
import render from "./entry.ssr";
import { serve } from "@hono/node-server";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

declare global {
  interface QwikCityPlatform extends PlatformNode {}
}

const distDir = join(fileURLToPath(import.meta.url), "..", "..", "dist");
const buildDir = join(distDir, "build");

const PORT = process.env.PORT ?? 3000;

const { router, notFound } = createQwikCity({
  render,
  qwikCityPlan,
  manifest,
});

const app = new Hono();

app.use(`/build`, serveStatic({ root: buildDir }));
app.use(`*`, serveStatic({ root: distDir }));

import type { IncomingMessage } from "http";
import type { ServerResponse } from "node:http";

app.use(`*`, async (c, next) => {
  return await router(
    c.req as unknown as IncomingMessage,
    c.res as unknown as ServerResponse<IncomingMessage>,
    next
  );
});

app.use(`*`, async (c, next) => {
  return await notFound(
    c.req as unknown as IncomingMessage,
    c.res as unknown as ServerResponse<IncomingMessage>,
    next
  );
});

serve(
  {
    fetch: app.fetch,
    port: PORT as number,
  },
  (info) => {
    console.log(`Server started: http://localhost:${info.port}/`);
  }
);
