import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";

// Mirrors the markup emitted by note_html in
// crates/provenance-cli/src/wiki/render/discussion.rs for a long note.
// The shape is pinned on the Rust side by
// long_discussion_notes_collapse_behind_their_derived_first_line.
const collapsedNote = `<!doctype html><html><body>
<div class="field-note">
<details class="fn-collapsible">
<summary><span class="fn-takeaway">Opening conclusion.</span><span class="fn-expand">Expand note</span></summary>
<div class="fn-content">
<p>Opening conclusion.</p>
<p>Supporting detail remains verbatim.</p>
</div>
</details>
</div>
</body></html>`;

test("collapsed note starts closed and expands on native summary activation", () => {
  const dom = new JSDOM(collapsedNote, { runScripts: "dangerously" });
  const details = dom.window.document.querySelector("details.fn-collapsible");
  assert.ok(details, "collapsible note renders a <details> element");
  assert.equal(details.open, false, "long note starts collapsed");

  const summary = details.querySelector("summary");
  assert.equal(
    summary,
    details.firstElementChild,
    "summary is the first child so the affordance is native",
  );
  summary.dispatchEvent(
    new dom.window.MouseEvent("click", { bubbles: true, cancelable: true }),
  );
  assert.equal(details.open, true, "clicking the summary expands the note");
});

test("collapse affordance needs no scripts or inline handlers", () => {
  const dom = new JSDOM(collapsedNote);
  const document = dom.window.document;
  assert.equal(document.querySelectorAll("script").length, 0);
  for (const element of document.querySelectorAll("*")) {
    for (const attribute of element.attributes) {
      assert.ok(
        !attribute.name.startsWith("on"),
        `unexpected inline handler ${attribute.name} on <${element.tagName}>`,
      );
    }
  }
});
