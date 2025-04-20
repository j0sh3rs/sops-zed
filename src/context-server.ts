import { spawnSync } from "node:child_process";
import * as readline from "node:readline";
import { JSONRPCServer } from "json-rpc-2.0";

// 1) Create your JSON‑RPC server and register methods:
const server = new JSONRPCServer();

server.addMethod("decrypt", ({ text }: { text: string }) => {
	const proc = spawnSync("sops", ["-d", "--input-type", "binary"], {
		input: Buffer.from(text, "utf8"),
		encoding: "utf8",
	});
	if (proc.error || proc.status !== 0) {
		throw new Error(proc.stderr || "sops decryption failed");
	}
	return { text: proc.stdout };
});

server.addMethod("encrypt", ({ text }: { text: string }) => {
	const proc = spawnSync("sops", ["-e", "--output-type", "binary"], {
		input: Buffer.from(text, "utf8"),
		encoding: "utf8",
	});
	if (proc.error || proc.status !== 0) {
		throw new Error(proc.stderr || "sops encryption failed");
	}
	return { text: proc.stdout };
});

// 2) Read lines from stdin, feed them to the server, and write responses:
const rl = readline.createInterface({
	input: process.stdin,
	output: process.stdout,
	terminal: true,
});

rl.on("line", async (line) => {
	try {
		const response = await server.receive(line);
		if (response) {
			// JSON‑RPC‑2.0 spec wants each response on its own line
			process.stdout.write(`${JSON.stringify(response)}\n`);
		}
	} catch (err: any) {
		// If an internal error occurs, you can log or ignore
		console.error("Internal RPC error:", err);
	}
});
