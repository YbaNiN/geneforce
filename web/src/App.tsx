import { useState, useCallback } from "react";
import { PlantInput } from "./components/PlantInput";
import { TargetPanel, type TargetCounts } from "./components/TargetPanel";
import { OptionsPanel } from "./components/OptionsPanel";
import { ResultCard } from "./components/ResultCard";
import { TwoStepCard } from "./components/TwoStepCard";
import { FormModal } from "./components/FormModal";
import { solve, WasmNotBuiltError } from "./lib/breeder";
import { FORM_DEFS, type FormKind } from "./lib/forms";
import type {
  CandidateOut,
  GeneChar,
  SolveResponse,
  TwoStepOut,
} from "./lib/types";

export default function App() {
  const [plants, setPlants] = useState<string[]>([]);
  const [target, setTarget] = useState<TargetCounts>({ G: 3, Y: 3 });
  const [maxDonors, setMaxDonors] = useState(4);
  const [allowTwoGen, setAllowTwoGen] = useState(false);

  const [results, setResults] = useState<CandidateOut[] | null>(null);
  const [twoStep, setTwoStep] = useState<TwoStepOut[]>([]);
  const [calculating, setCalculating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [highlighted, setHighlighted] = useState<Set<number>>(new Set());
  const [activeForm, setActiveForm] = useState<FormKind | null>(null);

  const addPlant = useCallback((genes: string) => {
    setPlants((prev) => [...prev, genes]);
  }, []);

  const removePlant = useCallback((index: number) => {
    setPlants((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleHighlight = useCallback(
    (plantStrings: string[]) => {
      if (plantStrings.length === 0) {
        setHighlighted(new Set());
        return;
      }
      const indices = new Set<number>();
      const used = new Set<number>();
      for (const ps of plantStrings) {
        const idx = plants.findIndex((p, i) => p === ps && !used.has(i));
        if (idx !== -1) {
          indices.add(idx);
          used.add(idx);
        }
      }
      setHighlighted(indices);
    },
    [plants],
  );

  async function handleCalculate() {
    setError(null);
    setCalculating(true);
    setResults(null);
    setTwoStep([]);
    try {
      const counts: [string, number][] = (
        Object.entries(target) as [GeneChar, number][]
      )
        .filter(([, n]) => n > 0)
        .map(([g, n]) => [g, n]);

      const response: SolveResponse = await solve({
        pool: plants,
        target: { counts },
        max_donors: maxDonors,
        top_n: 10,
        allow_two_gen: allowTwoGen,
      });

      setResults(response.candidates);
      setTwoStep(response.two_step);
      if (response.invalid.length > 0) {
        setError(
          `${response.invalid.length} planta(s) inválida(s) fueron ignoradas.`,
        );
      }
    } catch (e) {
      if (e instanceof WasmNotBuiltError) {
        setError(e.message);
      } else {
        setError(`Error al calcular: ${e instanceof Error ? e.message : e}`);
      }
    } finally {
      setCalculating(false);
    }
  }

  const canCalculate = plants.length >= 2 && !calculating;

  return (
    <div className="min-h-screen">
      <Header onOpenForm={setActiveForm} />

      <main className="max-w-5xl mx-auto px-4 pb-16">
        <div className="grid md:grid-cols-[1.3fr_1fr] gap-4">
          {/* Columna izquierda: plantas + resultados */}
          <div>
            <div className="section-label mb-2">Tus clones / semillas</div>
            <PlantInput
              plants={plants}
              onAdd={addPlant}
              onRemove={removePlant}
              highlightedIndices={highlighted}
            />

            {results !== null && (
              <div className="mt-6">
                <div className="section-label mb-2">
                  Resultados {results.length > 0 && `(${results.length})`}
                </div>
                {results.length === 0 ? (
                  <div className="panel p-6 text-center">
                    <p className="text-bone-muted text-sm">
                      No se encontró ninguna combinación de un paso que produzca
                      el objetivo con estas plantas.
                    </p>
                    <p className="text-bone-dim text-xs mt-2">
                      {allowTwoGen && twoStep.length > 0
                        ? "Pero hay rutas de dos pasos (GEN.2) más abajo."
                        : "Prueba a añadir más clones, subir el rango de circundantes, activar GEN.2, o ajustar el objetivo."}
                    </p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {results.map((c, i) => (
                      <ResultCard
                        key={i}
                        candidate={c}
                        rank={i + 1}
                        onHighlight={handleHighlight}
                      />
                    ))}
                  </div>
                )}

                {twoStep.length > 0 && (
                  <div className="mt-6">
                    <div className="section-label mb-2 flex items-center gap-2">
                      Rutas de dos pasos
                      <span className="text-[10px] px-1.5 py-0.5 bg-carbon-500 text-rust-light border border-rust/50 rounded-sm normal-case tracking-normal">
                        GEN.2
                      </span>
                    </div>
                    <div className="space-y-3">
                      {twoStep.map((r, i) => (
                        <TwoStepCard key={i} recipe={r} rank={i + 1} />
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Columna derecha: objetivo + opciones + acción */}
          <div className="space-y-4">
            <div>
              <div className="section-label mb-2">Objetivo (god clone)</div>
              <TargetPanel counts={target} onChange={setTarget} />
            </div>

            <div>
              <div className="section-label mb-2">Opciones</div>
              <OptionsPanel
                maxDonors={maxDonors}
                onMaxDonorsChange={setMaxDonors}
                allowTwoGen={allowTwoGen}
                onAllowTwoGenChange={setAllowTwoGen}
              />
            </div>

            <button
              onClick={handleCalculate}
              disabled={!canCalculate}
              className="btn-rust w-full py-3.5 text-sm rounded-sm"
            >
              {calculating ? "Calculando…" : "⚗ Calcular"}
            </button>

            {plants.length < 2 && (
              <p className="text-bone-dim text-xs text-center">
                Añade al menos 2 plantas para calcular.
              </p>
            )}

            {error && (
              <div className="panel p-3 border-gene-w/40">
                <p className="text-gene-w text-xs font-mono leading-relaxed">
                  {error}
                </p>
              </div>
            )}
          </div>
        </div>
      </main>

      <Footer />

      <FormModal kind={activeForm} onClose={() => setActiveForm(null)} />
    </div>
  );
}

function Header({ onOpenForm }: { onOpenForm: (kind: FormKind) => void }) {
  const formButtons = (Object.keys(FORM_DEFS) as FormKind[]).map((kind) => ({
    kind,
    label: FORM_DEFS[kind].label,
  }));

  return (
    <header className="max-w-5xl mx-auto px-4 pt-8 pb-6">
      <div className="flex flex-wrap items-center gap-3 pb-5 border-b border-steel">
        <div className="w-8 h-8 bg-rust flex items-center justify-center rotate-45 shrink-0">
          <span className="-rotate-45 text-carbon-900 font-bold text-lg">⌬</span>
        </div>
        <div>
          <h1 className="text-xl font-bold uppercase tracking-[0.15em] text-bone leading-none">
            GeneForge
          </h1>
          <p className="text-[11px] text-bone-muted tracking-wider uppercase font-mono mt-1">
            crossbreeding calculator
          </p>
        </div>
        <nav className="ml-auto flex flex-wrap gap-2">
          {formButtons.map((b) => (
            <button
              key={b.kind}
              onClick={() => onOpenForm(b.kind)}
              className="text-[11px] px-3 py-1.5 border border-steel-light text-bone-muted uppercase tracking-wide rounded-sm transition-colors hover:border-rust hover:text-rust-light"
            >
              {b.label}
            </button>
          ))}
        </nav>
      </div>
    </header>
  );
}

function Footer() {
  return (
    <footer className="max-w-5xl mx-auto px-4 py-8 border-t border-steel mt-8">
      <p className="text-bone-dim text-xs font-mono leading-relaxed">
        Calculadora no oficial para la mecánica de cruce de plantas de Rust.
        Genes verdes (G/Y/H) pesan 0.6, rojos (X/W) pesan 1.0. El cálculo es
        determinista salvo en empates, mostrados como slots en disputa.
      </p>
    </footer>
  );
}
