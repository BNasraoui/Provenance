import { spawn } from "node:child_process";

export interface EngineSettings {
  engine: string;
  repository: string;
  scope: string;
}

export async function invokeEngine<Result>(
  settings: EngineSettings,
  command: string,
  input?: unknown,
): Promise<Result> {
  const child = spawn(
    settings.engine,
    [
      "sdk",
      command,
      "--repo",
      settings.repository,
      "--scope",
      settings.scope,
      "--format",
      "json",
    ],
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
