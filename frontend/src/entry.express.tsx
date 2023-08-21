/*
 * WHAT IS THIS FILE?
 *
 * It's the entry point for the Express HTTP server when building for production.
 *
 * Learn more about Node.js server integrations here:
 * - https://qwik.builder.io/docs/deployments/node/
 *
 */
import {
  createQwikCity,
  type PlatformNode,
} from "@builder.io/qwik-city/middleware/node";
import qwikCityPlan from "@qwik-city-plan";
import { manifest } from "@qwik-client-manifest";
import render from "./entry.ssr";
import express from "express";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import * as Sentry from "@sentry/node";

declare global {
  interface QwikCityPlatform extends PlatformNode {}
}

// import compression from 'compression';

// Directories where the static assets are located
const distDir = join(fileURLToPath(import.meta.url), "..", "..", "dist");
const buildDir = join(distDir, "build");

// Allow for dynamic port
const PORT = process.env.PORT ?? 3000;

// Create the Qwik City Node middleware
const { router, notFound } = createQwikCity({ render, qwikCityPlan, manifest });

// Create the express server
// https://expressjs.com/
const app = express();

Sentry.init({
    dsn: "https://68e207c76f024ea6fd6a2db5e0101965@debug.villablanca.tech/7",
    integrations: [
        new Sentry.Integrations.Http({tracing:true}),
        new Sentry.Integrations.Express({app}),
    ],
    tracesSampleRate: 1.0,
});

app.disable("x-powered-by");

app.use(Sentry.Handlers.requestHandler({ip: true}));
app.use(Sentry.Handlers.tracingHandler());

// Enable gzip compression
// app.use(compression());

// Static asset handlers
// https://expressjs.com/en/starter/static-files.html
app.use(`/build`, express.static(buildDir, { immutable: true, maxAge: "1y" }));
app.use(express.static(distDir, { redirect: false }));

// Use Qwik City's page and endpoint request handler
app.use(router);

app.use(Sentry.Handlers.errorHandler());

// Use Qwik City's 404 handler
app.use(notFound);

// Start the express server
app.listen(PORT, () => {
  /* eslint-disable */
  console.log(`Server started: http://localhost:${PORT}/`);
});
