import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root,
  base: "./",
  plugins: [vue()],
  build: {
    outDir: resolve(root, "../../test/browser/vite-dist"),
    emptyOutDir: true,
  },
});
