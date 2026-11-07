import { createElement, useEffect, useRef, useState } from "react";
import { CorrplotError, renderMatrixToSvg } from "./renderer.js";

export { CorrplotError };

const EMPTY_OPTIONS = Object.freeze({});

export function useCorrplot(matrix, sceneOptions = EMPTY_OPTIONS, svgOptions = EMPTY_OPTIONS) {
  const [state, setState] = useState({
    svg: null,
    loading: true,
    error: null,
  });

  useEffect(() => {
    let active = true;
    setState({ svg: null, loading: true, error: null });

    renderMatrixToSvg(matrix, sceneOptions, svgOptions)
      .then(svg => {
        if (active) setState({ svg, loading: false, error: null });
      })
      .catch(error => {
        if (!active) return;
        const normalized = error instanceof Error ? error : new Error(String(error));
        setState({ svg: null, loading: false, error: normalized });
      });

    return () => {
      active = false;
    };
  }, [matrix, sceneOptions, svgOptions]);

  return state;
}

export function Corrplot({
  matrix,
  sceneOptions = EMPTY_OPTIONS,
  svgOptions = EMPTY_OPTIONS,
  className,
  style,
  ariaLabel = "Correlation plot",
  onError,
}) {
  const state = useCorrplot(matrix, sceneOptions, svgOptions);
  const lastNotifiedError = useRef(null);

  useEffect(() => {
    if (!state.error) {
      lastNotifiedError.current = null;
      return;
    }
    if (lastNotifiedError.current === state.error) return;

    lastNotifiedError.current = state.error;
    onError?.(state.error);
  }, [state.error, onError]);

  const { svg, loading, error } = state;
  const attributes = {
    className,
    style,
    role: error ? "alert" : "img",
    "aria-label": ariaLabel,
    "aria-busy": loading,
    "data-testid": "corrplot",
    "data-status": error ? "error" : loading ? "loading" : "ready",
  };

  if (svg) attributes.dangerouslySetInnerHTML = { __html: svg };
  return createElement(
    "div",
    attributes,
    svg ? undefined : error ? error.message : loading ? "Loading corrplot…" : undefined,
  );
}
