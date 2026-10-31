"use strict";

const native = require("./index.node");

class CorrplotError extends Error {
  constructor(diagnostic) {
    super(diagnostic.message);
    this.name = "CorrplotError";
    this.diagnostic = diagnostic;
  }
}

function unwrap(result, field) {
  if (result.diagnostic) {
    throw new CorrplotError(result.diagnostic);
  }
  return result[field];
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

function getRendererCapabilities() {
  return JSON.parse(native.getRendererCapabilities());
}

module.exports = {
  CorrplotError,
  validateMatrix,
  renderSceneSvg,
  getRendererCapabilities,
};
