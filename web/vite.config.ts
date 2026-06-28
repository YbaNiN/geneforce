import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// El plugin de WASM permite importar el módulo generado por wasm-pack
// directamente como un ES module. top-level-await es necesario porque la
// inicialización del módulo WASM es asíncrona.
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  base: "./",
  build: {
    target: "esnext",
  },
});
