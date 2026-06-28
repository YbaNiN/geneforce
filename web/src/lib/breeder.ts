// Wrapper sobre el módulo WASM generado por wasm-pack.
//
// El módulo WASM se genera ejecutando `npm run build:wasm`, que produce
// `src/wasm/geneforge_wasm.js` + `.wasm`. Aquí lo cargamos de forma perezosa
// y exponemos una API tipada y ergonómica al resto de la app.

import type {
  CrossResult,
  SolveRequest,
  SolveResponse,
} from "./types";

// El import del módulo WASM es dinámico para que la app cargue aunque el
// usuario aún no haya compilado el WASM (caso típico en primer arranque).
// Tipamos la forma del módulo manualmente porque los tipos generados viven
// en src/wasm tras la compilación.
interface GeneForgeWasm {
  default: () => Promise<unknown>;
  validate_plant: (s: string) => string | undefined;
  crossbreed: (center: string, donors: string[]) => CrossResult;
  solve: (request: SolveRequest) => SolveResponse;
}

let wasmModule: GeneForgeWasm | null = null;
let initPromise: Promise<GeneForgeWasm> | null = null;

export class WasmNotBuiltError extends Error {
  constructor() {
    super(
      "El módulo WASM no está compilado. Ejecuta `npm run build:wasm` antes de `npm run dev`.",
    );
    this.name = "WasmNotBuiltError";
  }
}

async function loadWasm(): Promise<GeneForgeWasm> {
  if (wasmModule) return wasmModule;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    let mod: GeneForgeWasm;
    try {
      // Import dinámico con ruta literal: Vite lo reconoce, empaqueta el módulo
      // WASM en assets/ y reescribe la ruta correctamente en producción.
      mod = (await import("../wasm/geneforge_wasm.js")) as unknown as GeneForgeWasm;
    } catch (e) {
      console.error("No se pudo cargar el módulo WASM:", e);
      throw new WasmNotBuiltError();
    }
    await mod.default();
    wasmModule = mod;
    return mod;
  })();

  return initPromise;
}

/** Pre-carga el módulo WASM. Útil para llamarlo al montar la app. */
export async function initBreeder(): Promise<void> {
  await loadWasm();
}

/** Valida una cadena de genes. Devuelve null si es válida, o el mensaje de error. */
export async function validatePlant(s: string): Promise<string | null> {
  const wasm = await loadWasm();
  const err = wasm.validate_plant(s);
  return err ?? null;
}

/** Cruza un centro con donantes y devuelve el resultado por slot. */
export async function crossbreed(
  center: string,
  donors: string[],
): Promise<CrossResult> {
  const wasm = await loadWasm();
  return wasm.crossbreed(center, donors);
}

/** Ejecuta el solver completo. */
export async function solve(request: SolveRequest): Promise<SolveResponse> {
  const wasm = await loadWasm();
  return wasm.solve(request);
}
