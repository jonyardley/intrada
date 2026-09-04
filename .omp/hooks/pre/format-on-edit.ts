// OMP does not run another harness's hooks, so the PostToolUse formatter in
// .claude/settings.json stopped firing at the harness move and agent edits
// began reaching CI unformatted. Keep the two in step.

import { execFile } from "node:child_process";
import { isAbsolute, resolve } from "node:path";
import { promisify } from "node:util";

const run = promisify(execFile);

// Hashline headers are [path#A1B2]; `MV dest` is what actually lands on disk.
const SECTION = /^\[([^\]\n#]+)#[0-9A-Fa-f]{4}\]\s*$/gm;
const MOVE = /^MV\s+(?:"([^"]+)"|(\S+))\s*$/gm;

interface ToolResultEvent {
	toolName: string;
	isError?: boolean;
	input?: Record<string, unknown>;
}

interface HookContext {
	cwd?: string;
}

interface HookAPI {
	on: (
		event: "tool_result",
		handler: (event: ToolResultEvent, ctx: HookContext) => Promise<void> | void,
	) => void;
	logger?: { debug?: (message: string) => void };
}

export function editedPaths(toolName: string, input: Record<string, unknown>): string[] {
	const out: string[] = [];
	if (toolName === "write") {
		const p = input.path;
		if (typeof p === "string") out.push(p);
	} else if (toolName === "edit") {
		const body = typeof input.input === "string" ? input.input : "";
		for (const m of body.matchAll(SECTION)) out.push(m[1]);
		for (const m of body.matchAll(MOVE)) out.push(m[1] ?? m[2]);
	}
	return out.filter((p) => !p.includes("://") && !p.includes("ios/generated/"));
}

async function format(file: string): Promise<void> {
	if (file.endsWith(".rs")) {
		await run("rustfmt", ["--edition", "2021", file]);
	} else if (file.endsWith(".swift")) {
		await run("swift", ["format", "--in-place", file]);
	}
}

export default function hook(pi: HookAPI): void {
	pi.on("tool_result", async (event, ctx) => {
		if (event.isError) return;
		if (event.toolName !== "edit" && event.toolName !== "write") return;

		const cwd = ctx?.cwd ?? process.cwd();
		for (const rel of editedPaths(event.toolName, event.input ?? {})) {
			if (!rel.endsWith(".rs") && !rel.endsWith(".swift")) continue;
			try {
				await format(isAbsolute(rel) ? rel : resolve(cwd, rel));
			} catch (err) {
				// Fail open: a missing or unhappy formatter must not fail a good edit.
				pi.logger?.debug?.(`format-on-edit skipped ${rel}: ${String(err)}`);
			}
		}
	});
}
