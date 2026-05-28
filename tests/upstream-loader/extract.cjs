"use strict";
const path = require("path");

require("./hijack.cjs");

try {
  process.env.TS_NODE_COMPILER_OPTIONS = JSON.stringify({
    module: "CommonJS",
    moduleResolution: "node",
    esModuleInterop: true,
    target: "ES2020",
  });
  const tsNodePath = require.resolve("ts-node/register/transpile-only", { paths: [process.cwd()] });
  require(tsNodePath);
} catch (e) {
}

const target = process.argv[2];
if (!target) {
  console.error("usage: extract.cjs <test-file>");
  process.exit(1);
}
require(path.resolve(target));
