import type { ExtensionContext } from "zed";

export function activate(ctx: ExtensionContext) {
	const server = ctx.startContextServer();

	// Auto-decrypt on open
	ctx.workspace.onDidOpenTextDocument(async (doc) => {
		if (shouldHandle(doc.uri)) {
			const res = await server.request("decrypt", { text: doc.getText() });
			doc.replaceWholeText(res.text);
		}
	});

	// Auto-encrypt on save
	ctx.workspace.onWillSaveTextDocument(async (evt) => {
		const doc = evt.document;
		if (shouldHandle(doc.uri)) {
			const res = await server.request("encrypt", { text: doc.getText() });
			evt.waitUntil(
				Promise.resolve([
					{
						range: doc.fullRange,
						text: res.text,
					},
				]),
			);
		}
	});

	// Manual commands
	ctx.commands.registerCommand("sops.decrypt", async () => {
		const doc = ctx.workspace.getActiveTextDocument();
		if (!doc || !shouldHandle(doc.uri)) return;
		const res = await server.request("decrypt", { text: doc.getText() });
		doc.replaceWholeText(res.text);
	});
	ctx.commands.registerCommand("sops.encrypt", async () => {
		const doc = ctx.workspace.getActiveTextDocument();
		if (!doc || !shouldHandle(doc.uri)) return;
		const res = await server.request("encrypt", { text: doc.getText() });
		doc.replaceWholeText(res.text);
	});
}

function shouldHandle(uri: string): boolean {
	return (
		(uri.includes("sops") && uri.endsWith(".yaml")) ||
		uri.endsWith(".enc") ||
		/\.env$/.test(uri)
	);
}
