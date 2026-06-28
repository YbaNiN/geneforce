import { useState } from "react";
import type { GeneChar } from "../lib/types";
import { GENES } from "../lib/types";
import { GeneBox } from "./GeneBox";

interface PlantInputProps {
  plants: string[];
  onAdd: (genes: string) => void;
  onRemove: (index: number) => void;
  onSelect?: (index: number) => void;
  highlightedIndices?: Set<number>;
}

export function PlantInput({
  plants,
  onAdd,
  onRemove,
  onSelect,
  highlightedIndices,
}: PlantInputProps) {
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);

  function validate(value: string): string | null {
    const cleaned = value.trim().toUpperCase();
    if (cleaned.length !== 6) return "Se necesitan exactamente 6 genes";
    for (const c of cleaned) {
      if (!GENES.includes(c as GeneChar)) return `Gen inválido: ${c}`;
    }
    return null;
  }

  function handleAdd() {
    const cleaned = draft.trim().toUpperCase();
    const err = validate(cleaned);
    if (err) {
      setError(err);
      return;
    }
    onAdd(cleaned);
    setDraft("");
    setError(null);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") handleAdd();
  }

  return (
    <div>
      <div className="panel p-3 mb-3 min-h-[80px]">
        {plants.length === 0 ? (
          <p className="text-bone-dim text-sm py-4 text-center">
            Añade los clones que tienes para empezar.
          </p>
        ) : (
          <div className="space-y-2">
            {plants.map((plant, i) => {
              const highlighted = highlightedIndices?.has(i);
              return (
                <div
                  key={`${plant}-${i}`}
                  className={`grid grid-cols-[24px_repeat(6,1fr)_28px] gap-1 items-center px-1 py-0.5 rounded-sm transition-colors ${
                    highlighted ? "bg-rust/15 ring-1 ring-rust/40" : ""
                  } ${onSelect ? "cursor-pointer hover:bg-carbon-600" : ""}`}
                  onClick={() => onSelect?.(i)}
                >
                  <span className="font-mono text-xs text-bone-muted">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  {plant.split("").map((g, j) => (
                    <GeneBox key={j} gene={g as GeneChar} size="sm" />
                  ))}
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemove(i);
                    }}
                    className="text-bone-dim hover:text-gene-w transition-colors text-sm"
                    aria-label={`Eliminar planta ${i + 1}`}
                  >
                    ✕
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="flex gap-2">
        <input
          value={draft}
          onChange={(e) => {
            setDraft(e.target.value);
            setError(null);
          }}
          onKeyDown={handleKeyDown}
          placeholder="GGGYYH"
          maxLength={6}
          spellCheck={false}
          className="flex-1 bg-carbon-700 border border-steel rounded-sm px-3 py-2 font-mono tracking-[0.2em] uppercase text-sm text-bone placeholder:text-bone-dim focus:outline-none focus:border-rust transition-colors"
        />
        <button onClick={handleAdd} className="btn-rust px-4 py-2 text-xs rounded-sm">
          + Añadir
        </button>
      </div>
      {error && <p className="text-gene-w text-xs mt-1.5 font-mono">{error}</p>}
    </div>
  );
}
