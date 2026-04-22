// Deno sidecar: 读 stdin JSON-lines，回写 stdout JSON-lines
import { readLines } from "https://deno.land/std@0.224.0/io/read_lines.ts";

async function main() {
  for await (const line of readLines(Deno.stdin)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const req = JSON.parse(trimmed);
      const res = { id: req.id ?? null, type: "pong", data: "ok" };
      console.log(JSON.stringify(res));
    } catch {
      console.log(JSON.stringify({ id: null, type: "error", data: "invalid json" }));
    }
  }
}

main();
