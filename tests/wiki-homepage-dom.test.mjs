import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";

import { measureMobileViewport, representativeHomepage } from "./support/homepage-fixture.mjs";

const { html, scale } = representativeHomepage();

function contentControls(document) {
  return [...document.querySelectorAll(".body-main input, .body-main button, .body-main a")];
}

function tabbableControls(document) {
  return [...document.querySelectorAll('a[href], input, button, select, [tabindex]:not([tabindex="-1"])')];
}

test("PR 45 scale remains bounded and keeps complete indexes off the homepage", () => {
  const document = new JSDOM(html).window.document;

  // The PR #45 corpus holds 12 domains, below the 20-row homepage cap, so
  // this asserts one row per authored domain; cap enforcement itself is
  // covered by homepage_caps_authored_domain_rows in the Rust suite.
  assert.equal(document.querySelectorAll("[data-homepage-domain-row]").length, scale.domains.length);
  assert.ok(document.querySelectorAll("[data-homepage-domain-row]").length <= 20);
  assert.ok(Buffer.byteLength(html) < 50_000);
  for (const count of [228, 165, 576, 7, 42]) {
    assert.ok(document.body.textContent.includes(String(count)), `missing representative count ${count}`);
  }
  assert.equal(document.querySelectorAll("[data-search-entry], .domain-records, .citation.gap").length, 0);
  for (const marker of Object.values(scale.off_homepage_markers)) {
    assert.ok(!html.includes(marker), `homepage inlined dedicated-page detail: ${marker}`);
  }
});

test("homepage entry points keep the ratified search, domains, unfinished order", () => {
  const document = new JSDOM(html).window.document;
  const main = document.querySelector(".body-main");
  const sections = [...main.children].filter((node) => node.matches("section"));

  assert.deepEqual(
    sections.map((section) => {
      if (section.querySelector('[role="search"]')) return "search";
      if (section.matches("[data-homepage-domains]")) return "domains";
      if (section.matches("[data-homepage-traceability]")) return "unfinished";
      return "unknown";
    }),
    ["search", "domains", "unfinished"],
  );
});

test("search and homepage links follow the content tab order and accept focus", () => {
  const dom = new JSDOM(html, { pretendToBeVisual: true, url: "https://wiki.test/" });
  const controls = contentControls(dom.window.document);
  const tabOrder = tabbableControls(dom.window.document);

  assert.deepEqual(
    controls.map((control) => control.matches("input, button")
      ? control.localName
      : `${control.pathname}${control.hash}`),
    [
      "input", "button",
      ...scale.domains.map((domain) => `/domains/#domain-${domain.id}`),
      "/domains/", "/unfinished/",
    ],
  );
  assert.ok(tabOrder.indexOf(controls[0]) < tabOrder.indexOf(controls[2]));
  assert.ok(tabOrder.indexOf(controls[2]) < tabOrder.indexOf(controls.at(-1)));
  for (const control of tabOrder) {
    assert.ok(!control.hasAttribute("tabindex") || Number(control.tabIndex) >= 0);
    control.focus();
    assert.equal(dom.window.document.activeElement, control);
  }
});

test("homepage has no horizontal overflow at the ratified 390x844 viewport", async () => {
  const measurement = await measureMobileViewport(html, 390, 844);

  assert.equal(measurement.innerWidth, 390);
  assert.equal(measurement.innerHeight, 844);
  assert.ok(measurement.scrollWidth <= measurement.innerWidth, JSON.stringify(measurement));
  assert.ok(measurement.entryTops[0] < measurement.entryTops[1]);
  assert.ok(measurement.entryTops[1] < measurement.entryTops[2]);
});
