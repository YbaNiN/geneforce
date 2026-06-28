import type { CandidateOut, GeneChar } from "../lib/types";
import { GeneBox, TieBox } from "./GeneBox";

interface ResultCardProps {
  candidate: CandidateOut;
  rank: number;
  onHighlight?: (plants: string[]) => void;
}

export function ResultCard({ candidate, rank, onHighlight }: ResultCardProps) {
  const pct = Math.round(candidate.success_probability * 100);
  const isPerfect = pct === 100;
  // GEN.1 si todos los donantes provienen del pool directo (asumido aquí);
  // la lógica de GEN.2 se marca cuando el solver lo indique en una iteración futura.
  const genLabel = "GEN.1";

  return (
    <div
      className="panel p-3.5 border-rust/60 hover:border-rust transition-colors"
      onMouseEnter={() => onHighlight?.([candidate.center, ...candidate.donors])}
      onMouseLeave={() => onHighlight?.([])}
    >
      <div className="flex justify-between items-center mb-2.5">
        <div className="flex items-center gap-2">
          <span className="text-[11px] text-bone-dim font-mono">#{rank}</span>
          <span className="text-[11px] px-2 py-0.5 bg-rust text-carbon-900 font-bold tracking-wide rounded-sm">
            {genLabel}
          </span>
        </div>
        <span
          className={`text-sm font-mono font-semibold ${
            isPerfect ? "text-gene-g" : "text-rust-light"
          }`}
        >
          {pct}% exacto
        </span>
      </div>

      <div className="grid grid-cols-6 gap-1.5 mb-3">
        {candidate.result.slots.map((slot, i) => {
          if (slot.certain) {
            return <GeneBox key={i} gene={slot.options[0].gene} size="lg" />;
          }
          return (
            <TieBox
              key={i}
              genes={slot.options.map((o) => o.gene as GeneChar)}
              size="lg"
            />
          );
        })}
      </div>

      <div className="text-xs text-bone-muted space-y-1 font-mono">
        <div>
          <span className="text-bone-dim">centro:</span>{" "}
          <span className="text-bone">{candidate.center}</span>
        </div>
        <div className="flex flex-wrap gap-x-2 gap-y-1">
          <span className="text-bone-dim">circundantes:</span>
          {candidate.donors.map((d, i) => (
            <span key={i} className="text-bone">
              {d}
            </span>
          ))}
        </div>
        {!isPerfect && (
          <div className="text-rust-light pt-0.5">
            Algunos slots en disputa — ver patrón rayado arriba
          </div>
        )}
      </div>
    </div>
  );
}
