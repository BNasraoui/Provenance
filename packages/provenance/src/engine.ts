import { spawn } from "node:child_process";

export interface EngineSettings {
  engine: string;
  repository?: string;
  scope: string;
}

interface EngineInfo {
  engine_version: string;
  protocol_version: number;
  state_schema_version: number;
  repository: string;
}

const SUPPORTED_PROTOCOL_VERSION = 1;
const handshakes = new Map<string, Promise<void>>();

export async function invokeEngine<Result>(
  settings: EngineSettings,
  command: string,
  input?: unknown,
): Promise<Result> {
  await compatibleEngine(settings);
  return invoke<Result>(settings, command, input);
}

// @provenance rule: rule_sdk_protocol_handshake
async function compatibleEngine(settings: EngineSettings): Promise<void> {
  let handshake = handshakes.get(settings.engine);
  if (handshake === undefined) {
    handshake = invoke<EngineInfo>(settings, "info").then((info) => {
      if (info.protocol_version !== SUPPORTED_PROTOCOL_VERSION) {
        throw new Error(
          `Provenance engine ${info.engine_version} uses protocol version ${info.protocol_version}; ` +
          `this SDK supports protocol version ${SUPPORTED_PROTOCOL_VERSION}`,
        );
      }
    });
    handshakes.set(settings.engine, handshake);
  }
  await handshake;
}

async function invoke<Result>(
  settings: EngineSettings,
  command: string,
  input?: unknown,
): Promise<Result> {
  const args = ["sdk", command];
  if (settings.repository !== undefined) {
    args.push("--repo", settings.repository);
  }
  if (command !== "info") {
    args.push("--scope", settings.scope);
  }
  args.push("--format", "json");
  const child = spawn(
    settings.engine,
    args,
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const completed = new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code) => resolve(code ?? 1));
  });
  child.stdin.end(input === undefined ? undefined : JSON.stringify(input));
  const code = await completed;
  const output = Buffer.concat(stdout).toString("utf8");
  const diagnostics = Buffer.concat(stderr).toString("utf8").trim();
  if (code !== 0) {
    throw new Error(
      `Provenance engine command \`${command}\` failed (${code}): ${diagnostics || output}`,
    );
  }
  try {
    return JSON.parse(output) as Result;
  } catch (error) {
    throw new Error(
      `Provenance engine command \`${command}\` returned invalid JSON`,
      { cause: error },
    );
  }
}
