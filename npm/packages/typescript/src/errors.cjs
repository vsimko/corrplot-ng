class CorrplotError extends Error {
  constructor(diagnostic) {
    super(diagnostic.message);
    this.name = "CorrplotError";
    this.diagnostic = Object.freeze({ ...diagnostic });
  }
}

module.exports = { CorrplotError };
