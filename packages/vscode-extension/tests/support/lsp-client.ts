import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import type { Subprocess } from "bun";

type Json = Record<string, unknown>;
type RequestHandler = (params: Json) => unknown;
type ServerProcess = Subprocess<"pipe", "pipe", "pipe">;

interface Waiter {
	predicate: (params: Json) => boolean;
	reject: (error: Error) => void;
	resolve: (params: Json) => void;
}

const isJson = (value: unknown): value is Json =>
	typeof value === "object" && value !== null && !Array.isArray(value);

/**
 * A minimal LSP connection over the stdio of a server process: just the
 * Content-Length framing and JSON-RPC, so the tests need no extra package.
 */
class LspConnection {
	readonly process: ServerProcess;
	/** Every notification received, in order. */
	readonly notifications: { method: string; params: Json }[] = [];
	/** Params of every server-to-client request received, by method. */
	readonly requests: { method: string; params: Json }[] = [];
	private buffer = Buffer.alloc(0);
	private nextId = 1;
	private readonly pending = new Map<
		number,
		{ resolve: (value: unknown) => void; reject: (error: Error) => void }
	>();
	private readonly handlers = new Map<string, RequestHandler>();
	private waiters: { method: string; waiter: Waiter }[] = [];
	readonly stderr: string[] = [];
	/** Private cache home, so the tests never read or write the real cache. */
	private readonly home: string;

	private constructor(command: string, home: string) {
		this.home = home;
		this.process = Bun.spawn([command], {
			env: {
				...process.env,
				HOME: home,
				LOCALAPPDATA: home,
				XDG_CACHE_HOME: home,
			},
			stderr: "pipe",
			stdin: "pipe",
			stdout: "pipe",
		});
		void this.watchExit();
		void this.pump(this.process.stdout, (chunk): void => {
			this.buffer = Buffer.concat([this.buffer, chunk]);
			this.drain();
		});
		void this.pump(this.process.stderr, (chunk): void => {
			this.stderr.push(chunk.toString());
		});
	}

	/** Starts the server with a private cache home. */
	static async start(command: string): Promise<LspConnection> {
		const home = await mkdtemp(join(tmpdir(), "versionlens-lsp-"));
		return new LspConnection(command, home);
	}

	/** Answers a server-to-client request. */
	onRequest(method: string, handler: RequestHandler): void {
		this.handlers.set(method, handler);
	}

	request(method: string, params: unknown): Promise<unknown> {
		const id = this.nextId++;
		const result = new Promise<unknown>((resolve, reject): void => {
			this.pending.set(id, { reject, resolve });
		});
		this.send({ id, jsonrpc: "2.0", method, params });
		return result;
	}

	notify(method: string, params: unknown): void {
		this.send({ jsonrpc: "2.0", method, params });
	}

	/**
	 * Resolves with the params of the first notification of `method` that
	 * matches, including ones that arrived before the call.
	 */
	waitFor(
		method: string,
		predicate: (params: Json) => boolean = (): boolean => true,
		timeoutMs = 4000,
	): Promise<Json> {
		const seen = this.notifications.find(
			(entry): boolean => entry.method === method && predicate(entry.params),
		);
		if (seen !== undefined) {
			return Promise.resolve(seen.params);
		}
		return new Promise<Json>((resolve, reject): void => {
			const timer = setTimeout((): void => {
				this.waiters = this.waiters.filter(
					(entry): boolean => entry.waiter.reject !== fail,
				);
				reject(
					new Error(
						`timed out waiting for ${method}; stderr: ${this.stderr.join("")}`,
					),
				);
			}, timeoutMs);
			const fail = (error: Error): void => {
				clearTimeout(timer);
				reject(error);
			};
			this.waiters.push({
				method,
				waiter: {
					predicate,
					reject: fail,
					resolve: (params: Json): void => {
						clearTimeout(timer);
						resolve(params);
					},
				},
			});
		});
	}

	async shutdown(): Promise<void> {
		const alive = (): boolean =>
			this.process.exitCode === null && this.process.signalCode === null;
		const timer = setTimeout((): void => {
			this.process.kill();
		}, 3000);
		try {
			if (alive()) {
				const killed = async (): Promise<never> => {
					await this.process.exited;
					throw new Error("killed");
				};
				await Promise.race([this.request("shutdown", null), killed()]).catch(
					(): void => undefined,
				);
				this.notify("exit", null);
			}
			if (alive()) {
				await this.process.exited;
			}
		} finally {
			clearTimeout(timer);
			await rm(this.home, { force: true, recursive: true });
		}
	}

	private async watchExit(): Promise<void> {
		const code = await this.process.exited;
		const error = new Error(
			`the server exited (code ${code}, signal ${this.process.signalCode}); stderr: ${this.stderr.join("")}`,
		);
		for (const pending of this.pending.values()) {
			pending.reject(error);
		}
		this.pending.clear();
		for (const { waiter } of this.waiters) {
			waiter.reject(error);
		}
		this.waiters = [];
	}

	private async pump(
		stream: ReadableStream<Uint8Array>,
		onChunk: (chunk: Buffer) => void,
	): Promise<void> {
		for await (const chunk of stream) {
			onChunk(Buffer.from(chunk));
		}
	}

	private send(message: Json): void {
		const body = Buffer.from(JSON.stringify(message));
		void this.process.stdin.write(
			Buffer.concat([
				Buffer.from(`Content-Length: ${body.length}\r\n\r\n`),
				body,
			]),
		);
		void this.process.stdin.flush();
	}

	private drain(): void {
		for (;;) {
			const separator = this.buffer.indexOf("\r\n\r\n");
			if (separator === -1) {
				return;
			}
			const header = this.buffer.subarray(0, separator).toString();
			const length = Number(/Content-Length:\s*(\d+)/iu.exec(header)?.[1]);
			const end = separator + 4 + length;
			if (this.buffer.length < end) {
				return;
			}
			const message: unknown = JSON.parse(
				this.buffer.subarray(separator + 4, end).toString(),
			);
			this.buffer = this.buffer.subarray(end);
			if (!isJson(message)) {
				throw new TypeError("the server sent a message that is not an object");
			}
			this.dispatch(message);
		}
	}

	private dispatch(message: Json): void {
		const { id, method } = message;
		const params = isJson(message["params"]) ? message["params"] : {};
		if (typeof method !== "string") {
			this.settle(id, message);
			return;
		}
		if (typeof id === "number" || typeof id === "string") {
			this.requests.push({ method, params });
			void this.answer(id, method, params);
			return;
		}
		this.notifications.push({ method, params });
		this.waiters = this.waiters.filter((entry): boolean => {
			if (entry.method === method && entry.waiter.predicate(params)) {
				entry.waiter.resolve(params);
				return false;
			}
			return true;
		});
	}

	private settle(id: unknown, message: Json): void {
		if (typeof id !== "number") {
			return;
		}
		const pending = this.pending.get(id);
		this.pending.delete(id);
		const error = message["error"];
		if (error !== undefined && error !== null) {
			pending?.reject(new Error(JSON.stringify(error)));
		} else {
			pending?.resolve(message["result"]);
		}
	}

	private async answer(
		id: number | string,
		method: string,
		params: Json,
	): Promise<void> {
		const handler = this.handlers.get(method);
		if (handler === undefined) {
			this.send({
				error: { code: -32_601, message: `method not found: ${method}` },
				id,
				jsonrpc: "2.0",
			});
			return;
		}
		try {
			const result = await handler(params);
			this.send({ id, jsonrpc: "2.0", result: result ?? null });
		} catch (error) {
			this.send({
				error: { code: -32_603, message: String(error) },
				id,
				jsonrpc: "2.0",
			});
		}
	}
}

export type { Json };
export { isJson, LspConnection };
