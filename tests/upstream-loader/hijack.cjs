// textlint-tester を hijack して tester.run() の引数を JSON dump する。
// CommonJS test (.js / .ts via ts-node) のどちらでも動く。
//
// 使い方: UNSLOP_CASES_OUT=out.json node extract.cjs <path/to/test-file>

"use strict";
const fs = require("fs");
const path = require("path");
const Module = require("module");

const cases = [];

// 入力ケースを正規化する。string → { text } に、optionsCount 等の余計なフィールドは捨てる。
function normalizeValid(c) {
  if (typeof c === "string") return { text: c };
  return { text: c.text, options: c.options ?? null };
}
function normalizeInvalid(c) {
  return {
    text: c.text,
    options: c.options ?? null,
    errors: (c.errors || []).map((e) => ({
      message: e.message,
      messageId: e.messageId,
      line: e.line,
      column: e.column,
      severity: e.severity,
      ruleId: e.ruleId,
    })),
  };
}

class FakeTester {
  run(name, rule, suite) {
    cases.push({
      name,
      valid: (suite.valid || []).map(normalizeValid),
      invalid: (suite.invalid || []).map(normalizeInvalid),
    });
  }
}

// textlint-tester の export 形状は版で違うため複数形式を用意する。
const fakeExports = FakeTester;
fakeExports.default = FakeTester;
fakeExports.TextLintTester = FakeTester;

const origResolve = Module._resolveFilename;
Module._resolveFilename = function (request, parent, ...rest) {
  if (request === "textlint-tester") return __filename;
  if (typeof request === "string" && (request.startsWith(".") || request.startsWith("/"))) {
    try {
      return origResolve.call(this, request, parent, ...rest);
    } catch (e) {
      for (const ext of [".ts", ".tsx", ".js", ".mjs", ".cjs"]) {
        try {
          return origResolve.call(this, request + ext, parent, ...rest);
        } catch (_) {}
      }
      for (const ext of [".ts", ".js"]) {
        try {
          return origResolve.call(this, request + "/index" + ext, parent, ...rest);
        } catch (_) {}
      }
      throw e;
    }
  }
  return origResolve.call(this, request, parent, ...rest);
};
const origLoad = Module._load;
Module._load = function (request, parent, ...rest) {
  if (request === "textlint-tester") return fakeExports;
  return origLoad.call(this, request, parent, ...rest);
};

// mocha グローバルの最小 stub。tester.run() を describe 内で呼ぶ test も少数あるため。
function runFn(fn) {
  try {
    const r = fn && fn.length === 0 ? fn() : undefined;
    if (r && typeof r.then === "function") r.catch(() => {});
  } catch (_) {}
}
global.describe = (name, fn) => runFn(fn);
global.context = global.describe;
global.suite = global.describe;
global.it = (name, fn) => runFn(fn);
global.test = global.it;
global.before = global.beforeEach = global.after = global.afterEach = () => {};

module.exports = fakeExports;

process.on("exit", () => {
  const out = process.env.UNSLOP_CASES_OUT;
  if (!out) return;
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, JSON.stringify(cases, null, 2));
  process.stderr.write(`[hijack] wrote ${cases.length} suite(s) to ${out}\n`);
});
