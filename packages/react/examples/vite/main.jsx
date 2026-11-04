import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { Corrplot } from "@corrplot-ng/react";

function validMatrix(revision) {
  return {
    labels: revision % 2 === 0 ? ["alpha", "beta"] : ["alpha", "gamma"],
    rows: 2,
    columns: 2,
    values: revision % 2 === 0
      ? [1, 0.65, 0.65, 1]
      : [1, -0.35, -0.35, 1],
    kind: "correlation",
  };
}

function Demo() {
  const [revision, setRevision] = useState(0);
  const [mounted, setMounted] = useState(true);
  const [invalid, setInvalid] = useState(false);
  const matrix = invalid
    ? {
        labels: ["alpha", "beta"],
        rows: 2,
        columns: 1,
        values: [1, 0.5],
        kind: "correlation",
      }
    : validMatrix(revision);

  return (
    <section>
      <button id="update" onClick={() => { setInvalid(false); setRevision(value => value + 1); }}>
        Update matrix
      </button>
      <button id="toggle" onClick={() => setMounted(value => !value)}>
        Toggle plot
      </button>
      <button id="invalid" onClick={() => setInvalid(true)}>
        Render invalid matrix
      </button>
      <button id="recover" onClick={() => setInvalid(false)}>
        Recover plot
      </button>
      <output id="revision">{revision}</output>
      {mounted && (
        <Corrplot
          matrix={matrix}
          sceneOptions={{
            cellMethod: "circle",
            title: "React integration",
            description: `React smoke revision ${revision}`,
          }}
          svgOptions={{ width: 240, height: 240 }}
          ariaLabel="React integration smoke figure"
          onError={error => {
            const output = document.querySelector("#error-message");
            if (output) output.textContent = error.message;
          }}
        />
      )}
    </section>
  );
}

createRoot(document.querySelector("#root")).render(
  <React.StrictMode>
    <Demo />
  </React.StrictMode>,
);
