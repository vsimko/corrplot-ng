"use strict";

const native = require("./index.node");

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
  if (result.diagnostic) {
    throw new CorrplotError(result.diagnostic);
  }
  return field === undefined ? result : result[field];
}

function validateMatrix(input) {
  const result = native.validateMatrix(input);
  if (!result.valid) {
    throw new CorrplotError(result.diagnostic);
  }
  return true;
}

function renderSceneSvg(scene, options) {
  return unwrap(native.renderSceneSvg(scene, options), "svg");
}

function renderSceneRgba(scene, options) {
  return unwrap(native.renderSceneRgba(scene, options));
}

function getRendererCapabilities() {
  return JSON.parse(native.getRendererCapabilities());
}

module.exports = {
  CorrplotError,
  validateMatrix,
  renderSceneSvg,
  renderSceneRgba,
  getRendererCapabilities,
};
