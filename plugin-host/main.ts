// Plugin Host — Deno sidecar
// Reads stdin JSON-lines, dispatches to plugin modules, writes stdout JSON-lines.

import { readLines } from "https://deno.land/std@0.224.0/io/read_lines.ts";

interface PluginManifest {
  id: string;
  name: string;
  version: string;
  main: string;
  permissions?: string[];
  features?: PluginFeature[];
}

interface PluginFeature {
  code: string;
  explain: string;
  cmds?: string[];
}

interface Request {
  id: string | null;
  type: string;
  pluginId?: string;
  action?: string;
  path?: string;
  payload?: unknown;
  data?: unknown;
}

// Map of pluginId -> { module export object }
const plugins = new Map<string, Record<string, (...args: unknown[]) => unknown | Promise<unknown>>>();

const HOME = Deno.env.get("HOME") ?? Deno.env.get("USERPROFILE") ?? "";
const PLUGINS_ROOT = `${HOME}/.open-tools/plugins`;

function respond(id: string | null, type: string, data: unknown, pluginId?: string) {
  const res: Record<string, unknown> = { id, type, data };
  if (pluginId) res.pluginId = pluginId;
  console.log(JSON.stringify(res));
}

async function loadPlugin(req: Request): Promise<void> {
  const pluginId = req.pluginId ?? req.path?.split("/").pop() ?? "unknown";
  const pluginPath = req.path ?? `${PLUGINS_ROOT}/${pluginId}`;

  try {
    const manifestText = await Deno.readTextFile(`${pluginPath}/rubick.json`);
    const manifest: PluginManifest = JSON.parse(manifestText);

    // Request permissions dynamically
    const perms = manifest.permissions ?? [];
    const permOpts: Deno.PermissionOptions = {};
    if (perms.includes("read") || perms.includes("fs")) {
      permOpts.read = true;
    }
    if (perms.includes("write") || perms.includes("fs-write")) {
      permOpts.write = true;
    }
    if (perms.includes("net") || perms.includes("network")) {
      permOpts.net = true;
    }
    if (perms.includes("env") || perms.includes("environment")) {
      permOpts.env = true;
    }
    if (perms.includes("run") || perms.includes("shell")) {
      permOpts.run = true;
    }
    if (perms.includes("ffi")) {
      permOpts.ffi = true;
    }

    const status = await Deno.permissions.request(permOpts);
    if (status.state !== "granted") {
      respond(req.id, "plugin-error", `Permissions denied for ${manifest.id}`, manifest.id);
      return;
    }

    // Dynamic import the plugin's main module
    const entryPoint = `${pluginPath}/${manifest.main}`;
    const mod = await import(entryPoint);

    const exports: Record<string, (...args: unknown[]) => unknown | Promise<unknown>> = {};
    if (mod.default && typeof mod.default === "object") {
      Object.assign(exports, mod.default);
    }
    // Also collect named exports that are functions
    for (const [key, val] of Object.entries(mod)) {
      if (typeof val === "function") {
        exports[key] = val as (...args: unknown[]) => unknown | Promise<unknown>;
      }
    }

    plugins.set(manifest.id, exports);
    respond(req.id, "plugin-loaded", "ok", manifest.id);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    respond(req.id, "plugin-error", msg, pluginId);
  }
}

async function invokePlugin(req: Request): Promise<void> {
  const { pluginId, action } = req;
  if (!pluginId || !action) {
    respond(req.id, "plugin-error", "missing pluginId or action");
    return;
  }

  const plugin = plugins.get(pluginId);
  if (!plugin) {
    respond(req.id, "plugin-error", `plugin '${pluginId}' not loaded`, pluginId);
    return;
  }

  const fn = plugin[action];
  if (typeof fn !== "function") {
    respond(req.id, "plugin-error", `action '${action}' not found in plugin '${pluginId}'`, pluginId);
    return;
  }

  try {
    const result = await fn(req.payload);
    respond(req.id, "plugin-result", result, pluginId);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    respond(req.id, "plugin-error", msg, pluginId);
  }
}

async function ping(req: Request): Promise<void> {
  respond(req.id, "pong", req.data ?? "ok");
}

async function handleRequest(raw: string): Promise<void> {
  let req: Request;
  try {
    req = JSON.parse(raw);
  } catch {
    respond(null, "error", "invalid json");
    return;
  }

  switch (req.type) {
    case "load-plugin":
      await loadPlugin(req);
      break;
    case "invoke":
      await invokePlugin(req);
      break;
    case "ping":
      await ping(req);
      break;
    default:
      respond(req.id, "error", `unknown type: ${req.type}`);
  }
}

async function main(): Promise<void> {
  for await (const line of readLines(Deno.stdin)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    await handleRequest(trimmed);
  }
}

main();
