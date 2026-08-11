import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";

const scaleUrl = new URL("../fixtures/pr-45-homepage-scale.json", import.meta.url);
const snapshotUrl = new URL(
  "../../crates/provenance-cli/src/wiki/render/tests/snapshots/provenance__wiki__render__tests__homepage__pr_45_scale_homepage.snap",
  import.meta.url,
);
const stylesheetUrl = new URL(
  "../../crates/provenance-cli/src/wiki/theme/provenance-wiki.css",
  import.meta.url,
);

function renderedSnapshot() {
  return readFileSync(snapshotUrl, "utf8").replace(/^---\n[\s\S]*?\n---\n/, "");
}

export function representativeHomepage() {
  const scale = JSON.parse(readFileSync(scaleUrl, "utf8"));
  return { html: renderedSnapshot(), scale };
}

export async function measureMobileViewport(html, width, height) {
  const stylesheet = readFileSync(stylesheetUrl, "utf8");
  let report;
  let resolveReport;
  let rejectReport;
  const reported = new Promise((resolve, reject) => {
    resolveReport = resolve;
    rejectReport = reject;
  });
  const reporter = `<script>requestAnimationFrame(function () { requestAnimationFrame(function () {
    fetch('/report', {method: 'POST', body: JSON.stringify({
      innerWidth: innerWidth, innerHeight: innerHeight,
      scrollWidth: document.documentElement.scrollWidth,
      entryTops: [document.querySelector('[role="search"]'),
        document.querySelector('[data-homepage-domains]'),
        document.querySelector('[data-homepage-traceability]')]
        .map(function (entry) { return entry.getBoundingClientRect().top; })
    })});
  }); });</script>`;
  const browserHtml = html.replace("</body>", `${reporter}</body>`);
  const viewportHtml = `<!doctype html><iframe title="viewport" src="/homepage"
    style="border:0;width:${width}px;height:${height}px"></iframe>`;
  const server = createServer((request, response) => {
    if (request.method === "POST" && request.url === "/report") {
      let body = "";
      request.setEncoding("utf8");
      request.on("data", (chunk) => { body += chunk; });
      request.on("end", () => {
        try {
          report = JSON.parse(body);
          response.end("ok");
          resolveReport(report);
        } catch (error) {
          rejectReport(error);
        }
      });
      return;
    }
    if (request.url === "/assets/provenance-wiki.css") {
      response.setHeader("content-type", "text/css");
      response.end(stylesheet);
      return;
    }
    response.setHeader("content-type", "text/html");
    response.end(request.url === "/homepage" ? browserHtml : viewportHtml);
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  const profileParent = join(process.cwd(), "target");
  mkdirSync(profileParent, { recursive: true });
  const profile = mkdtempSync(join(profileParent, "homepage-firefox-"));
  const browser = spawn("firefox", [
    "--headless", "--no-remote", "--profile", profile, "--width", "800", "--height", "900",
    `http://127.0.0.1:${port}/`,
  ]);
  let stderr = "";
  browser.stderr.on("data", (chunk) => { stderr += chunk; });
  // A cold CI runner can take well over 20s to boot Firefox the first time;
  // the assertion budget is startup, not rendering, so give it room.
  const timeout = setTimeout(() => rejectReport(new Error(`Firefox report timed out: ${stderr}`)), 90_000);
  browser.once("error", rejectReport);
  browser.once("exit", (code) => {
    if (!report) rejectReport(new Error(`Firefox exited ${code}: ${stderr}`));
  });
  try {
    return await reported;
  } finally {
    clearTimeout(timeout);
    browser.kill();
    server.close();
    rmSync(profile, { force: true, recursive: true });
  }
}
