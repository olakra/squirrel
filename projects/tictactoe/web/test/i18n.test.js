import { test } from "node:test";
import assert from "node:assert/strict";
import { formatMessage, loadMessages, translator } from "../www/lib/i18n.js";

test("formatMessage fills named placeholders", () => {
  assert.equal(formatMessage("Row {row}, column {column}", { row: 2, column: 3 }), "Row 2, column 3");
});

test("formatMessage leaves unknown placeholders visible", () => {
  assert.equal(formatMessage("Hello {name}", {}), "Hello {name}");
});

test("formatMessage returns text without placeholders unchanged", () => {
  assert.equal(formatMessage("Reset"), "Reset");
});

test("loadMessages returns the parsed catalogue", async () => {
  const fakeFetch = async (url) => {
    assert.equal(url, "i18n/en.json");
    return { ok: true, json: async () => ({ "action.reset": "Reset" }) };
  };
  const messages = await loadMessages(fakeFetch, "i18n/en.json");
  assert.deepEqual(messages, { "action.reset": "Reset" });
});

test("loadMessages throws when the catalogue cannot be fetched", async () => {
  const fakeFetch = async () => ({ ok: false, status: 404 });
  await assert.rejects(loadMessages(fakeFetch, "i18n/en.json"), /404/);
});

test("translator looks up a key and fills its placeholders", () => {
  const t = translator({ "status.won": "{mark} wins" });
  assert.equal(t("status.won", { mark: "X" }), "X wins");
});

test("translator throws on a missing key so gaps fail loudly", () => {
  const t = translator({});
  assert.throws(() => t("status.missing"), /status\.missing/);
});
