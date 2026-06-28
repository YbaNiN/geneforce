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
      // El path en variable + comentario vite-ignore evita que el bundler intente
      // resolver el módulo en build-time; se resuelve solo en runtime, cuando el
      // WASM ya ha sido compilado con `npm run build:wasm`.
      const wasmPath = "../wasm/geneforge_wasm.js";
      mod = (await import(/* @vite-ignore */ wasmPath)) as GeneForgeWasm;
    } catch {
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
