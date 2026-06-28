import type { CandidateOut, GeneChar, TwoStepOut } from "../lib/types";
import { GeneBox, TieBox } from "./GeneBox";

function ResultStrip({ candidate }: { candidate: CandidateOut }) {
  return (
    <div className="grid grid-cols-6 gap-1.5">
      {candidate.result.slots.map((slot, i) =>
        slot.certain ? (
          <GeneBox key={i} gene={slot.options[0].gene} size="md" />
        ) : (
          <TieBox
            key={i}
            genes={slot.options.map((o) => o.gene as GeneChar)}
            size="md"
          />
        ),
      )}
    </div>
  );
}

function StepBlock({
  step,
  label,
  produces,
}: {
  step: CandidateOut;
  label: string;
  produces: string;
}) {
  return (
    <div>
      <div className="flex items-center justify-between mb-1.5">
        <span className="text-[11px] uppercase tracking-wide text-bone-muted font-semibold">
          {label}
        </span>
        <span className="text-[11px] font-mono text-bone-dim">
          produce <span className="text-bone">{produces}</span>
        </span>
      </div>
      <ResultStrip candidate={step} />
      <div className="text-[11px] text-bone-muted font-mono mt-1.5">
        <span className="text-bone-dim">centro:</span> {step.center}{" "}
        <span className="text-bone-dim">· circundantes:</span>{" "}
        {step.donors.join(", ")}
      </div>
    </div>
  );
}

interface TwoStepCardProps {
  recipe: TwoStepOut;
  rank: number;
}

export function TwoStepCard({ recipe, rank }: TwoStepCardProps) {
  const pct = Math.round(recipe.total_probability * 100);
  const isPerfect = pct === 100;

  return (
    <div className="panel p-3.5 border-rust/40">
      <div className="flex justify-between items-center mb-3">
        <div className="flex items-center gap-2">
          <span className="text-[11px] text-bone-dim font-mono">#{rank}</span>
          <span className="text-[11px] px-2 py-0.5 bg-carbon-500 text-rust-light border border-rust/50 font-bold tracking-wide rounded-sm">
            GEN.2
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

      <div className="space-y-3">
        <StepBlock
          step={recipe.step1}
          label="Paso 1 — fabricar puente"
          produces={recipe.bridge}
        />

        <div className="flex items-center gap-2 text-bone-dim">
          <div className="flex-1 h-px bg-steel" />
          <span className="text-[10px] uppercase tracking-widest">luego</span>
          <div className="flex-1 h-px bg-steel" />
        </div>

        <StepBlock
          step={recipe.step2}
          label="Paso 2 — cruce final"
          produces="objetivo"
        />
      </div>

      <p className="text-[11px] text-bone-dim mt-3 leading-relaxed">
        Primero cruza el Paso 1 para obtener{" "}
        <span className="font-mono text-bone-muted">{recipe.bridge}</span>, que aún
        no tienes. Luego úsalo en el Paso 2 para alcanzar tu objetivo.
      </p>
    </div>
  );
}
