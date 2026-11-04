<script setup>
import { computed, ref } from "vue";
import { Corrplot, useCorrplot } from "@corrplot-ng/vue";

const revision = ref(0);
const mounted = ref(true);
const invalid = ref(false);
const errorCount = ref(0);

const matrix = computed(() => invalid.value
  ? {
      labels: ["alpha", "beta"],
      rows: 2,
      columns: 1,
      values: [1, 0.5],
      kind: "correlation",
    }
  : {
      labels: revision.value % 2 === 0 ? ["alpha", "beta"] : ["alpha", "gamma"],
      rows: 2,
      columns: 2,
      values: revision.value % 2 === 0
        ? [1, 0.65, 0.65, 1]
        : [1, -0.35, -0.35, 1],
      kind: "correlation",
    });

const sceneOptions = computed(() => ({
  cellMethod: "circle",
  title: "Vue integration",
  description: "Vue smoke revision " + revision.value,
}));

const svgOptions = Object.freeze({ width: 240, height: 240 });
const hookState = useCorrplot(matrix, sceneOptions, svgOptions);

function onError(error) {
  errorCount.value += 1;
  const output = document.querySelector("#error-message");
  if (output) output.textContent = error.message;
}
</script>

<template>
  <section>
    <button id="update" @click="() => { invalid = false; revision += 1; }">
      Update matrix
    </button>
    <button id="toggle" @click="mounted = !mounted">
      Toggle plot
    </button>
    <button id="invalid" @click="invalid = true">
      Render invalid matrix
    </button>
    <button id="recover" @click="invalid = false">
      Recover plot
    </button>
    <output id="revision">{{ revision }}</output>
    <output id="error-count">{{ errorCount }}</output>
    <output
      id="hook-status"
      :data-status="hookState.loading ? 'loading' : hookState.error ? 'error' : 'ready'"
      :data-svg-length="hookState.svg?.length ?? 0"
    >
      {{ hookState.loading ? "loading" : hookState.error ? "error" : "ready" }}
    </output>
    <Corrplot
      v-if="mounted"
      :matrix="matrix"
      :scene-options="sceneOptions"
      :svg-options="svgOptions"
      aria-label="Vue integration smoke figure"
      @error="onError"
    />
    <p id="error-message" role="status"></p>
  </section>
</template>
