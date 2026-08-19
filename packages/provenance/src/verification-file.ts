import { fileURLToPath } from "node:url";

// Which file a verification runs in. A test can state it, and on runtimes that
// report the calling frame the SDK reads it from the stack. Bun does not always
// report it: JavaScriptCore takes proper tail calls, so `test("...", () =>
// rule.verify(key, callback))` replaces the calling frame with the called one
// and leaves a stack of SDK frames alone. No stack format can recover a frame
// the runtime removed, so such a call must say where it lives.

/** The part of a verify option object that says which file the call runs in. */
export interface StatedFile {
  readonly file?: string;
  readonly url?: string;
}

const moduleFile = fileURLToPath(import.meta.url);

export function verificationFile(
  key: string,
  options: StatedFile | undefined,
  sdkFiles: readonly string[],
): string {
  const file = statedFile(options) ?? callingFile(new Error().stack, sdkFiles);
  if (file === undefined) {
    throw new Error(
      `Provenance could not tell which file verification \`${key}\` runs in, because this ` +
      "runtime does not report the calling file. State it in the call: " +
      `verify(${JSON.stringify(key)}, callback, import.meta). ` +
      "An explicit `{ file: import.meta.path }` works too.",
    );
  }
  return file;
}

/**
 * The file an options object states, from `import.meta` or from an explicit
 * `file`. A module URL wins, because it is the one property every runtime
 * spells the same way; Bun's `import.meta.file` holds a bare file name.
 */
export function statedFile(options: StatedFile | undefined): string | undefined {
  const url = options?.url;
  if (typeof url === "string" && url.startsWith("file:")) {
    return fileURLToPath(url);
  }
  const file = options?.file;
  if (typeof file !== "string" || file.trim() === "") {
    return undefined;
  }
  return file.startsWith("file:") ? fileURLToPath(file) : file;
}

/** The first file in the stack that is not part of the SDK, if the runtime reports one. */
export function callingFile(
  stack: string | undefined,
  sdkFiles: readonly string[],
): string | undefined {
  const skipped = [moduleFile, ...sdkFiles];
  for (const line of stack?.split("\n").slice(1) ?? []) {
    const openParenthesis = line.lastIndexOf("(");
    const framed = openParenthesis >= 0
      ? line.slice(openParenthesis + 1, line.endsWith(")") ? -1 : undefined)
      : line.trim().split(/\s+/).at(-1);
    const match = framed?.match(/^(.*):\d+:\d+$/);
    if (match?.[1] === undefined) continue;
    const file = match[1].startsWith("file:") ? fileURLToPath(match[1]) : match[1];
    if (!skipped.includes(file)) {
      return file;
    }
  }
  return undefined;
}
