import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import test from "node:test";
import { JSDOM, VirtualConsole } from "jsdom";

const script = readFileSync(
  new URL("../crates/provenance-cli/src/wiki/theme/search.js", import.meta.url),
  "utf8",
);
assert.ok(script, "the DOM suite must execute the shipped SEARCH_SCRIPT");

mkdirSync(resolve("target"), { recursive: true });
const workspace = mkdtempSync(resolve("target/wiki-search-dom-"));
const binary = resolve("target/debug/provenance");
let emptySearchHtml;
let populatedSearchHtml;

test.before(() => {
  execFileSync("cargo", ["build", "-p", "provenance-cli"]);
  emptySearchHtml = buildSearchPage("empty");
  populatedSearchHtml = buildSearchPage("populated", {
    scope: "default",
    sources: [],
    domains: [],
    requirements: [
      {
        schema_version: 1,
        scope_id: "default",
        id: "req_invoice",
        statement: "Invoice participant",
        status: "active",
        source_refs: [],
      },
      {
        schema_version: 1,
        scope_id: "default",
        id: "req_other",
        statement: "Unrelated requirement",
        status: "active",
        source_refs: [],
      },
    ],
    resolutions: [],
    rules: [],
    edges: [],
    threads: [],
    messages: [],
  });
});

test.after(() => rmSync(workspace, { recursive: true, force: true }));

function buildSearchPage(name, state) {
  const repo = join(workspace, `${name}-repo`);
  const output = join(workspace, `${name}-site`);
  execFileSync(binary, [
    "init",
    "--path",
    repo,
    "--scope",
    "default",
    "--path-prefix",
    ".",
  ]);
  if (state) {
    const input = join(workspace, `${name}.json`);
    writeFileSync(input, JSON.stringify(state));
    execFileSync(binary, [
      "import",
      "--repo",
      repo,
      "--scope",
      "default",
      "--input",
      input,
      "--format",
      "json",
    ]);
  }
  execFileSync(binary, ["wiki", "build", "--repo", repo, "--out", output]);
  return readFileSync(join(output, "search", "index.html"), "utf8");
}

function page(entries = "") {
  return `<!doctype html><html><body>
    <input id="wiki-search" type="search">
    <p id="search-summary"></p>
    <ol id="search-results">${entries}</ol>
  </body></html>`;
}

function run(html, url = "https://wiki.test/search/") {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on("jsdomError", (error) => errors.push(error));
  const dom = new JSDOM(html, {
    runScripts: "outside-only",
    url,
    virtualConsole,
  });
  try {
    dom.window.eval(script);
  } catch (error) {
    errors.push(error);
  }
  return { dom, errors };
}

function runRendered(html, url = "https://wiki.test/search/") {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on("jsdomError", (error) => errors.push(error));
  const dom = new JSDOM(html, {
    beforeParse(window) {
      window.matchMedia = () => ({ matches: false });
    },
    runScripts: "dangerously",
    url,
    virtualConsole,
  });
  return { dom, errors };
}

test("empty search index has a stable, executable DOM", () => {
  const { dom, errors } = runRendered(emptySearchHtml);

  assert.deepEqual(errors, []);
  assert.equal(dom.window.document.querySelectorAll("[data-search-entry]").length, 0);
  assert.equal(
    dom.window.document.querySelector("#search-summary").textContent,
    "Type one or more words to search titles and statements.",
  );
});

test("rendered search HTML and the shipped script share one DOM contract", () => {
  const { dom, errors } = runRendered(
    populatedSearchHtml,
    "https://wiki.test/search/?q=invoice%20participant",
  );
  const visible = [...dom.window.document.querySelectorAll("[data-search-entry]:not([hidden])")];

  assert.deepEqual(errors, []);
  assert.equal(dom.window.document.querySelector("#search-summary").textContent, "1 result");
  assert.equal(visible.length, 1);
  assert.equal(visible[0].querySelector("a").getAttribute("href"), "/requirements/req_invoice/");
});

test("a partial search DOM is ignored instead of crashing", () => {
  const { errors } = run('<input id="wiki-search" type="search">');
  assert.deepEqual(errors, []);
});

test("URL hydration runs the shipped matching, ranking, and safe DOM filtering", () => {
  const entries = `
    <li data-search-entry data-id="statement" data-search-title="Settlement" data-search-statement="Invoice participant">statement</li>
    <li data-search-entry data-id="title" data-search-title="Invoice participant" data-search-statement="Settlement">title</li>
    <li data-search-entry data-id="other" data-search-title="Unrelated" data-search-statement="Text">other</li>`;
  const { dom, errors } = run(
    page(entries),
    "https://wiki.test/search/?q=INVOICE%20participant",
  );
  const { document } = dom.window;

  assert.deepEqual(errors, []);
  assert.equal(document.querySelector("#wiki-search").value, "INVOICE participant");
  assert.equal(document.querySelector("#search-summary").textContent, "2 results");
  assert.deepEqual(
    [...document.querySelectorAll("[data-search-entry]:not([hidden])")].map(
      (item) => item.dataset.id,
    ),
    ["title", "statement"],
  );
  assert.equal(document.querySelector('[data-id="other"]').hidden, true);
  assert.equal(new URL(dom.window.location.href).searchParams.get("q"), "INVOICE participant");
});

test("input events update results and the query URL", () => {
  const entries = `
    <li data-search-entry data-id="one" data-search-title="One" data-search-statement="Alpha">one</li>
    <li data-search-entry data-id="two" data-search-title="Two" data-search-statement="Beta">two</li>`;
  const { dom, errors } = run(page(entries));
  const input = dom.window.document.querySelector("#wiki-search");

  input.value = "Beta";
  input.dispatchEvent(new dom.window.Event("input", { bubbles: true }));

  assert.deepEqual(errors, []);
  assert.equal(dom.window.document.querySelector("#search-summary").textContent, "1 result");
  assert.equal(dom.window.document.querySelector('[data-id="one"]').hidden, true);
  assert.equal(dom.window.document.querySelector('[data-id="two"]').hidden, false);
  assert.equal(new URL(dom.window.location.href).searchParams.get("q"), "Beta");
});
