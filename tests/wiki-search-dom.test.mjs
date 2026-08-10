import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { JSDOM, VirtualConsole } from "jsdom";

const script = readFileSync(
  new URL("../crates/provenance-cli/src/wiki/theme/search.js", import.meta.url),
  "utf8",
);

function fixture(entries = "") {
  return `<!doctype html><html><body>
    <input id="wiki-search" type="search"><p id="search-summary"></p><ol id="search-results">
    ${entries}
    </ol>
  </body></html>`;
}

function run(html, url = "https://wiki.test/search/") {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on("jsdomError", (error) => errors.push(error));
  const dom = new JSDOM(html, { runScripts: "outside-only", url, virtualConsole });
  try {
    dom.window.eval(script);
  } catch (error) {
    errors.push(error);
  }
  return { dom, errors };
}

test("empty DOM fixture is stable", () => {
  const { dom, errors } = run(fixture());
  assert.deepEqual(errors, []);
  assert.equal(
    dom.window.document.querySelector("#search-summary").textContent,
    "Type one or more words to search titles and statements.",
  );
});

test("partial DOM fixture exits without crashing", () => {
  const { errors } = run('<input id="wiki-search" type="search">');
  assert.deepEqual(errors, []);
});

test("populated DOM fixture uses case-insensitive all-term matching", () => {
  const entries = `
    <li data-search-entry data-id="one" data-search-title="Invoice participant" data-search-statement="Settlement rule">one</li>
    <li data-search-entry data-id="two" data-search-title="Invoice" data-search-statement="Unrelated">two</li>`;
  const { dom, errors } = run(fixture(entries));
  const input = dom.window.document.querySelector("#wiki-search");
  input.value = "INVOICE participant";
  input.dispatchEvent(new dom.window.Event("input", { bubbles: true }));

  assert.deepEqual(errors, []);
  assert.equal(dom.window.document.querySelector('[data-id="one"]').hidden, false);
  assert.equal(dom.window.document.querySelector('[data-id="two"]').hidden, true);
  assert.equal(dom.window.document.querySelector("#search-summary").textContent, "1 result");
});

test("zero matches are explained with a complete sentence", () => {
  const entries = `
    <li data-search-entry data-search-title="Invoice participant" data-search-statement="Settlement rule">one</li>`;
  const { dom, errors } = run(fixture(entries));
  const input = dom.window.document.querySelector("#wiki-search");
  input.value = "missing words";
  input.dispatchEvent(new dom.window.Event("input", { bubbles: true }));

  assert.deepEqual(errors, []);
  assert.equal(
    dom.window.document.querySelector("#search-summary").textContent,
    "No requirements or rules match this search.",
  );
});

test("hydrated DOM fixture ranks titles first with stable ties and preserves q", () => {
  const entries = `
    <li data-search-entry data-id="statement" data-search-title="Settlement" data-search-statement="Invoice participant">statement</li>
    <li data-search-entry data-id="title-first" data-search-title="Invoice participant details" data-search-statement="Settlement">title first</li>
    <li data-search-entry data-id="title-tie" data-search-title="Invoice participant summary" data-search-statement="Settlement">title tie</li>`;
  const { dom, errors } = run(
    fixture(entries),
    "https://wiki.test/search/?q=INVOICE%20participant",
  );
  const visible = [...dom.window.document.querySelectorAll("[data-search-entry]:not([hidden])")];

  assert.deepEqual(errors, []);
  assert.equal(dom.window.document.querySelector("#wiki-search").value, "INVOICE participant");
  assert.deepEqual(visible.map((item) => item.dataset.id), ["title-first", "title-tie", "statement"]);
  assert.equal(new URL(dom.window.location.href).searchParams.get("q"), "INVOICE participant");
});

test("a decision is found by its title or position", () => {
  const entries = `
    <li data-search-entry data-id="decision" data-search-title="Choose invoice grouping" data-search-statement="Adopt one invoice per participant">decision</li>
    <li data-search-entry data-id="rule" data-search-title="Group invoice lines" data-search-statement="Keep claim portions together">rule</li>`;
  const { dom, errors } = run(fixture(entries));
  const input = dom.window.document.querySelector("#wiki-search");

  for (const query of ["Choose invoice", "one invoice per participant"]) {
    input.value = query;
    input.dispatchEvent(new dom.window.Event("input", { bubbles: true }));
    assert.equal(dom.window.document.querySelector('[data-id="decision"]').hidden, false);
    assert.equal(dom.window.document.querySelector('[data-id="rule"]').hidden, true);
  }
  assert.deepEqual(errors, []);
});
