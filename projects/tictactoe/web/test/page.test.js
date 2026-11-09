import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join, dirname } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const indexHtml = readFileSync(join(here, "..", "www", "index.html"), "utf8");

// Text between tags, with script and style blocks removed. Only whitespace may remain.
function visibleText(html) {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, "")
    .replace(/<style[\s\S]*?<\/style>/gi, "")
    .replace(/<head[\s\S]*?<\/head>/gi, "")
    .replace(/<[^>]*>/g, "")
    .trim();
}

test("index.html has no hard-coded user-visible text", () => {
  assert.equal(visibleText(indexHtml), "");
});

test("index.html links the stylesheet and the module script", () => {
  assert.match(indexHtml, /<link[^>]+href="styles\.css"/);
  assert.match(indexHtml, /<script[^>]+type="module"[^>]+src="main\.js"/);
});

test("the status region is a polite live region", () => {
  assert.match(indexHtml, /id="status"[^>]*aria-live="polite"/);
});
