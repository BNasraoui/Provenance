import { readFileSync, realpathSync } from "node:fs";
import { dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";
import * as ts from "typescript";

import type { ImplementationDeclaration } from "./protocol.js";

export interface CallSite {
  readonly file: string;
  readonly line: number;
  readonly column?: number;
}

export type ImplementationTarget =
  | ((...args: never[]) => unknown)
  | (abstract new (...args: never[]) => unknown);

const moduleFile = fileURLToPath(import.meta.url);

export function captureImplementationReference(
  excludedFiles: readonly string[],
): ImplementationDeclaration {
  const sites = parseCallSites(new Error().stack ?? "");
  const excluded = [moduleFile, ...excludedFiles].map(runtimeModuleVariants);
  const site = sites.find(({ file }) => !excluded.some((files) => files.has(file)));
  if (site === undefined) {
    throw new Error(
      "implementedBy could not find a readable call site; bundled and minified specs are not supported",
    );
  }
  return resolveImplementationReferenceAt(site);
}

export function parseCallSites(stack: string): CallSite[] {
  const sites: CallSite[] = [];
  for (const line of stack.split("\n").slice(1)) {
    const framed = stackFrameLocation(line);
    if (framed === undefined) continue;
    const match = framed.match(/^(.*):(\d+):(\d+)$/) ?? framed.match(/^(.*):(\d+)$/);
    if (match?.[1] === undefined || match[2] === undefined) continue;
    const file = stackFile(match[1]);
    if (file === undefined) continue;
    sites.push({
      file,
      line: Number(match[2]),
      ...(match[3] === undefined ? {} : { column: Number(match[3]) }),
    });
  }
  return sites;
}

function runtimeModuleVariants(file: string): Set<string> {
  const files = new Set([file]);
  for (const output of ["dist", ".test-dist"]) {
    const marker = `/${output}/`;
    if (file.includes(marker)) {
      files.add(file.replace(marker, "/src/").replace(/\.js$/, ".ts"));
    }
    if (file.includes("/src/")) {
      files.add(file.replace("/src/", marker).replace(/\.ts$/, ".js"));
    }
  }
  return files;
}

export function resolveImplementationReferenceAt(site: CallSite): ImplementationDeclaration {
  const source = readCallSite(site.file);
  const sourceFile = ts.createSourceFile(
    site.file,
    source,
    ts.ScriptTarget.Latest,
    true,
    scriptKind(site.file),
  );
  const calls = implementedByCallsAt(sourceFile, site.line);
  if (calls.length !== 1) {
    const detail = calls.length === 0 ? "no" : "more than one";
    throw new Error(
      `implementedBy found ${detail} implementedBy call at ${site.file}:${site.line}; put each declaration call on its own readable line`,
    );
  }
  const argument = calls[0]!.arguments[0];
  if (argument === undefined) {
    throw unsupportedExpression(site);
  }
  const imported = importedReference(sourceFile, argument);
  if (imported === undefined) {
    throw unsupportedExpression(site);
  }
  const resolved = resolveModule(imported.module, site.file);
  return Object.freeze({ file: resolved, symbol: imported.symbol });
}

function stackFrameLocation(line: string): string | undefined {
  const trimmed = line.trim().replace(/^at\s+/, "");
  const open = trimmed.lastIndexOf("(");
  return open >= 0
    ? trimmed.slice(open + 1, trimmed.endsWith(")") ? -1 : undefined)
    : trimmed.split(/\s+/).at(-1);
}

function stackFile(value: string): string | undefined {
  if (value.startsWith("file:")) {
    try {
      return fileURLToPath(value);
    } catch {
      return undefined;
    }
  }
  return value.startsWith("/") || /^[A-Za-z]:[\\/]/.test(value) ? value : undefined;
}

function readCallSite(file: string): string {
  try {
    return readFileSync(file, "utf8");
  } catch (error) {
    throw new Error(`could not read implementedBy call site ${file}`, { cause: error });
  }
}

function scriptKind(file: string): ts.ScriptKind {
  switch (extname(file).toLowerCase()) {
    case ".ts":
    case ".mts":
    case ".cts":
      return ts.ScriptKind.TS;
    case ".tsx":
      return ts.ScriptKind.TSX;
    case ".jsx":
      return ts.ScriptKind.JSX;
    default:
      return ts.ScriptKind.JS;
  }
}

function implementedByCallsAt(source: ts.SourceFile, line: number): ts.CallExpression[] {
  const matches: ts.CallExpression[] = [];
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) && isImplementedBy(node.expression)) {
      const start = source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1;
      const end = source.getLineAndCharacterOfPosition(node.end).line + 1;
      if (start <= line && line <= end) matches.push(node);
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return matches;
}

function isImplementedBy(expression: ts.LeftHandSideExpression): boolean {
  return ts.isPropertyAccessExpression(expression) && expression.name.text === "implementedBy";
}

interface ImportedReference {
  readonly module: string;
  readonly symbol: string;
}

function importedReference(
  source: ts.SourceFile,
  expression: ts.Expression,
): ImportedReference | undefined {
  if (ts.isIdentifier(expression)) {
    return namedImport(source, expression.text);
  }
  if (
    ts.isPropertyAccessExpression(expression) &&
    !ts.isPrivateIdentifier(expression.name) &&
    ts.isIdentifier(expression.expression)
  ) {
    return namespaceImport(source, expression.expression.text, expression.name.text);
  }
  return undefined;
}

function namedImport(source: ts.SourceFile, local: string): ImportedReference | undefined {
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (!bindings || !ts.isNamedImports(bindings)) continue;
    const imported = bindings.elements.find(({ name }) => name.text === local);
    if (imported !== undefined) {
      return {
        module: statement.moduleSpecifier.text,
        symbol: imported.propertyName?.text ?? imported.name.text,
      };
    }
  }
  return undefined;
}

function namespaceImport(
  source: ts.SourceFile,
  local: string,
  symbol: string,
): ImportedReference | undefined {
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings) && bindings.name.text === local) {
      return { module: statement.moduleSpecifier.text, symbol };
    }
  }
  return undefined;
}

function resolveModule(specifier: string, containingFile: string): string {
  const options = compilerOptions(containingFile);
  const resolved = ts.resolveModuleName(specifier, containingFile, options, ts.sys).resolvedModule;
  if (resolved === undefined || resolved.isExternalLibraryImport) {
    throw new Error(
      `implementedBy could not resolve local module ${JSON.stringify(specifier)} from ${containingFile}`,
    );
  }
  return realpathSync(resolved.resolvedFileName);
}

function compilerOptions(containingFile: string): ts.CompilerOptions {
  const config = ts.findConfigFile(dirname(containingFile), ts.sys.fileExists);
  if (config === undefined) {
    return {
      allowJs: true,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
    };
  }
  const loaded = ts.readConfigFile(config, ts.sys.readFile);
  if (loaded.error !== undefined) {
    throw new Error(`implementedBy could not read TypeScript config ${config}`);
  }
  return ts.parseJsonConfigFileContent(loaded.config, ts.sys, dirname(config)).options;
}

function unsupportedExpression(site: CallSite): Error {
  return new Error(
    `implementedBy at ${site.file}:${site.line} requires a direct named import or namespace member; calls, conditionals, computed members, and local values are not durable identities`,
  );
}
