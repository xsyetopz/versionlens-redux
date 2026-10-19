import { type ChildProcess, spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

type Json = Record<string, unknown>;
type RequestHandler = (params: never) => unknown;

interface Waiter {
  predicate: (params: never) => boolean;
  reject: (error: Error) => void;
  resolve: (params: never) => void;
}

/**
 * A minimal LSP connection over the stdio of a server process: just the
 * Content-Length framing and JSON-RPC, so the tests need no extra package.
 */
class LspConnection {
  readonly process: ChildProcess;
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
  private readonly home = mkdtempSync(join(tmpdir(), "versionlens-lsp-"));

  constructor(command: string) {
    this.process = spawn(command, [], {
      env: {
        ...process.env,
        HOME: this.home,
        LOCALAPPDATA: this.home,
        XDG_CACHE_HOME: this.home,
      },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.process.once("exit", (code, signal): void => {
      const error = new Error(
        `the server exited (code ${code}, signal ${signal}); stderr: ${this.stderr.join("")}`,
      );
      for (const pending of this.pending.values()) {
        pending.reject(error);
      }
      this.pending.clear();
      for (const { waiter } of this.waiters) {
        waiter.reject(error);
      }
      this.waiters = [];
    });
    this.process.stdout?.on("data", (chunk: Buffer): void => {
      this.buffer = Buffer.concat([this.buffer, chunk]);
      this.drain();
    });
    this.process.stderr?.on("data", (chunk: Buffer): void => {
      this.stderr.push(chunk.toString());
    });
  }

  /** Answers a server-to-client request. */
  onRequest(method: string, handler: RequestHandler): void {
    this.handlers.set(method, handler);
  }

  request<T = unknown>(method: string, params: unknown): Promise<T> {
    const id = this.nextId++;
    const result = new Promise<T>((resolve, reject): void => {
      this.pending.set(id, {
        reject,
        resolve: resolve as (value: unknown) => void,
      });
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
  waitFor<T extends Json = Json>(
    method: string,
    predicate: (params: T) => boolean = (): boolean => true,
    timeoutMs = 4000,
  ): Promise<T> {
    const seen = this.notifications.find(
      (entry): boolean =>
        entry.method === method && predicate(entry.params as T),
    );
    if (seen) {
      return Promise.resolve(seen.params as T);
    }
    return new Promise<T>((resolve, reject): void => {
      const timer = setTimeout((): void => {
        this.waiters = this.waiters.filter(
          (entry) => entry.waiter.reject !== fail,
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
          predicate: predicate as (params: never) => boolean,
          reject: fail,
          resolve: ((params: T): void => {
            clearTimeout(timer);
            resolve(params);
          }) as (params: never) => void,
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
        const killed = new Promise<never>((_, reject): void => {
          this.process.once("exit", (): void => reject(new Error("killed")));
        });
        await Promise.race([this.request("shutdown", null), killed]).catch(
          (): void => undefined,
        );
        this.notify("exit", null);
      }
      if (alive()) {
        await new Promise<void>((resolve): void => {
          this.process.once("exit", (): void => resolve());
        });
      }
    } finally {
      clearTimeout(timer);
      rmSync(this.home, { force: true, recursive: true });
    }
  }

  private send(message: Json): void {
    const body = Buffer.from(JSON.stringify(message));
    this.process.stdin?.write(
      Buffer.concat([
        Buffer.from(`Content-Length: ${body.length}\r\n\r\n`),
        body,
      ]),
    );
  }

  private drain(): void {
    for (;;) {
      const separator = this.buffer.indexOf("\r\n\r\n");
      if (separator < 0) {
        return;
      }
      const header = this.buffer.subarray(0, separator).toString();
      const length = Number(/Content-Length:\s*(\d+)/i.exec(header)?.[1]);
      const end = separator + 4 + length;
      if (this.buffer.length < end) {
        return;
      }
      const message = JSON.parse(
        this.buffer.subarray(separator + 4, end).toString(),
      ) as Json;
      this.buffer = this.buffer.subarray(end);
      this.dispatch(message);
    }
  }

  private dispatch(message: Json): void {
    const { id, method, params } = message as {
      id?: number | string;
      method?: string;
      params?: Json;
    };
    if (method === undefined) {
      const pending = this.pending.get(id as number);
      this.pending.delete(id as number);
      if (message["error"]) {
        pending?.reject(new Error(JSON.stringify(message["error"])));
      } else {
        pending?.resolve(message["result"]);
      }
      return;
    }
    if (id !== undefined) {
      this.requests.push({ method, params: params ?? {} });
      this.answer(id, method, params);
      return;
    }
    this.notifications.push({ method, params: params ?? {} });
    this.waiters = this.waiters.filter((entry): boolean => {
      if (
        entry.method === method &&
        entry.waiter.predicate((params ?? {}) as never)
      ) {
        entry.waiter.resolve((params ?? {}) as never);
        return false;
      }
      return true;
    });
  }

  private answer(id: number | string, method: string, params?: Json): void {
    const handler = this.handlers.get(method);
    if (!handler) {
      this.send({
        error: { code: -32_601, message: `method not found: ${method}` },
        id,
        jsonrpc: "2.0",
      });
      return;
    }
    Promise.resolve(handler((params ?? {}) as never)).then(
      (result): void => {
        this.send({ id, jsonrpc: "2.0", result: result ?? null });
      },
      (error: unknown): void => {
        this.send({
          error: { code: -32_603, message: String(error) },
          id,
          jsonrpc: "2.0",
        });
      },
    );
  }
}

export type { Json };
export { LspConnection };
