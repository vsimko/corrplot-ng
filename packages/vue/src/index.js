import {
  defineComponent,
  h,
  onBeforeUnmount,
  onMounted,
  readonly,
  shallowRef,
  toValue,
  watch,
} from "vue";
import { CorrplotError, renderMatrixToSvg } from "./renderer.js";

export { CorrplotError };

const EMPTY_OPTIONS = Object.freeze({});

/**
 * Render a matrix reactively inside a Vue setup() function.
 *
 * Each argument may be a value, ref/computed ref, or getter. Reactive object
 * changes are observed deeply; replace typed arrays rather than mutating their
 * indexed values in place.
 */
export function useCorrplot(
  matrix,
  sceneOptions = EMPTY_OPTIONS,
  svgOptions = EMPTY_OPTIONS,
) {
  const svg = shallowRef(null);
  const loading = shallowRef(true);
  const error = shallowRef(null);

  let generation = 0;
  let stopWatch;

  onMounted(() => {
    stopWatch = watch(
      () => [
        toValue(matrix),
        toValue(sceneOptions),
        toValue(svgOptions),
      ],
      async ([currentMatrix, currentSceneOptions, currentSvgOptions]) => {
        const request = ++generation;
        svg.value = null;
        loading.value = true;
        error.value = null;

        try {
          const result = await renderMatrixToSvg(
            currentMatrix,
            currentSceneOptions,
            currentSvgOptions,
          );
          if (request !== generation) return;
          svg.value = result;
          loading.value = false;
        } catch (cause) {
          if (request !== generation) return;
          error.value = cause instanceof Error ? cause : new Error(String(cause));
          loading.value = false;
        }
      },
      {
        deep: true,
        flush: "post",
        immediate: true,
      },
    );
  });

  onBeforeUnmount(() => {
    generation += 1;
    stopWatch?.();
  });

  return {
    svg: readonly(svg),
    loading: readonly(loading),
    error: readonly(error),
  };
}

export const Corrplot = defineComponent({
  name: "Corrplot",
  inheritAttrs: false,
  props: {
    matrix: {
      type: Object,
      required: true,
    },
    sceneOptions: {
      type: Object,
      default: () => EMPTY_OPTIONS,
    },
    svgOptions: {
      type: Object,
      default: () => EMPTY_OPTIONS,
    },
    ariaLabel: {
      type: String,
      default: "Correlation plot",
    },
  },
  emits: ["error"],
  setup(props, { attrs, emit }) {
    const state = useCorrplot(
      () => props.matrix,
      () => props.sceneOptions,
      () => props.svgOptions,
    );

    watch(
      state.error,
      nextError => {
        if (nextError) emit("error", nextError);
      },
      { flush: "post" },
    );

    return () => {
      const currentSvg = state.svg.value;
      const currentError = state.error.value;
      const isLoading = state.loading.value;

      return h(
        "div",
        {
          ...attrs,
          role: currentError ? "alert" : "img",
          "aria-label": props.ariaLabel,
          "aria-busy": isLoading,
          "data-testid": "corrplot",
          "data-status": currentError ? "error" : isLoading ? "loading" : "ready",
          ...(currentSvg ? { innerHTML: currentSvg } : {}),
        },
        currentSvg
          ? undefined
          : currentError
            ? currentError.message
            : isLoading
              ? "Loading corrplot…"
              : undefined,
      );
    };
  },
});
