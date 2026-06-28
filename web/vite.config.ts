import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// El plugin de WASM permite importar el módulo generado por wasm-pack
// directamente como un ES module. top-level-await es necesario porque la
// inicialización del módulo WASM es asíncrona.
//
// `base` debe coincidir con el nombre del repositorio en GitHub Pages, ya que
// la app se sirve desde https://usuario.github.io/geneforce/ (no desde la raíz).
// Si despliegas en otro sitio (raíz de un dominio propio), cámbialo a "/".
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  base: "/geneforce/",
  build: {
    target: "esnext",
  },
});
