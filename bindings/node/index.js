"use strict";

const platformPackages = {
  "linux-x64": "@corrplot-ng/native-node-linux-x64-gnu",
  "darwin-x64": "@corrplot-ng/native-node-darwin-x64",
  "darwin-arm64": "@corrplot-ng/native-node-darwin-arm64",
  "win32-x64": "@corrplot-ng/native-node-win32-x64-msvc",
};

function loadNative() {
  const packageName = platformPackages[process.platform + "-" + process.arch];
  if (packageName) {
    try { return require(packageName); }
    catch (error) {
      if (error.code !== "MODULE_NOT_FOUND" || !error.message.includes(packageName)) throw error;
    }
  }
  try { return require("./index.node"); }
  catch (error) {
    if (error.code !== "MODULE_NOT_FOUND") throw error;
    throw new Error(
      "No prebuilt corrplot-ng native binary is installed for " + process.platform + "-" + process.arch + ". " +
      "Supported platforms: Linux x64 glibc, macOS x64/arm64, and Windows x64. " +
      "In a source checkout, run npm run build with Rust installed.",
      { cause: error },
    );
  }
}

const native = loadNative();

function normalizeDiagnostic(diagnostic) {
  return {
    schema_version: diagnostic.schemaVersion,
    code: diagnostic.code,
    severity: diagnostic.severity,
    stage: diagnostic.stage,
    message: diagnostic.message,
    ...(diagnostic.path === undefined ? {} : { path: diagnostic.path }),
    ...(diagnostic.row === undefined ? {} : { row: diagnostic.row }),
    ...(diagnostic.column === undefined ? {} : { column: diagnostic.column }),
  };
}

class CorrplotError extends Error {
  constructor(diagnostic) {
    const normalized = normalizeDiagnostic(diagnostic);
    super(normalized.message);
    this.name = "CorrplotError";
    this.diagnostic = normalized;
  }
}

function unwrap(result, field) {
  if (result.diagnostic) throw new CorrplotError(result.diagnostic);
  return field === undefined ? result : result[field];
}

function validateMatrix(input) {
  const result = native.validateMatrix(input);
  if (!result.valid) throw new CorrplotError(result.diagnostic);
  return true;
}

function renderSceneSvg(scene, options) { return unwrap(native.renderSceneSvg(scene, options), "svg"); }
function renderSceneRgba(scene, options) { return unwrap(native.renderSceneRgba(scene, options)); }
function renderMatrixSvg(sceneV1Json, method = "circle") { return unwrap(native.renderMatrixSvg(sceneV1Json, method), "svg"); }
function renderMatrixRgba(sceneV1Json, method = "circle", options) { return unwrap(native.renderMatrixRgba(sceneV1Json, method, options)); }
function getRendererCapabilities() { return JSON.parse(native.getRendererCapabilities()); }

module.exports = { CorrplotError, validateMatrix, renderSceneSvg, renderSceneRgba, renderMatrixSvg, renderMatrixRgba, getRendererCapabilities };
