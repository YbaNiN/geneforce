import type { GeneChar } from "../lib/types";
import { GREEN_GENES, GENE_LABELS, TARGET_PRESETS } from "../lib/types";

export type TargetCounts = Partial<Record<GeneChar, number>>;

interface TargetPanelProps {
  counts: TargetCounts;
  onChange: (counts: TargetCounts) => void;
}

const GENE_TEXT_CLASS: Record<string, string> = {
  G: "text-gene-g",
  Y: "text-gene-y",
  H: "text-gene-h",
};

export function TargetPanel({ counts, onChange }: TargetPanelProps) {
  const total = GREEN_GENES.reduce((sum, g) => sum + (counts[g] ?? 0), 0);

  function setGene(gene: GeneChar, value: number) {
    onChange({ ...counts, [gene]: Math.max(0, Math.min(6, value)) });
  }

  function applyPreset(presetCounts: TargetCounts) {
    onChange({ ...presetCounts });
  }

  function matchesPreset(presetCounts: TargetCounts): boolean {
    return GREEN_GENES.every((g) => (counts[g] ?? 0) === (presetCounts[g] ?? 0));
  }

  return (
    <div className="panel p-3.5">
      <div className="flex gap-2 mb-3">
        {TARGET_PRESETS.map((preset) => {
          const active = matchesPreset(preset.counts);
          return (
            <button
              key={preset.id}
              onClick={() => applyPreset(preset.counts)}
              title={preset.description}
              className={`px-3 py-1.5 text-xs font-mono rounded-sm border transition-colors ${
                active
                  ? "bg-rust text-carbon-900 border-rust font-bold"
                  : "bg-carbon-800 text-bone-muted border-steel hover:border-steel-light"
              }`}
            >
              {preset.label}
            </button>
          );
        })}
      </div>

      <div className="grid grid-cols-3 gap-2.5">
        {GREEN_GENES.map((gene) => (
          <div key={gene}>
            <label
              className={`text-xs uppercase block mb-1 ${GENE_TEXT_CLASS[gene]}`}
              title={GENE_LABELS[gene]}
            >
              {gene}
            </label>
            <input
              type="number"
              min={0}
              max={6}
              value={counts[gene] ?? 0}
              onChange={(e) => setGene(gene, parseInt(e.target.value) || 0)}
              className={`w-full bg-carbon-800 border border-steel rounded-sm py-1.5 text-center font-mono font-bold focus:outline-none focus:border-rust transition-colors ${GENE_TEXT_CLASS[gene]}`}
            />
          </div>
        ))}
      </div>

      <p
        className={`text-xs mt-2.5 font-mono ${
          total === 6 ? "text-bone-muted" : "text-gene-w"
        }`}
      >
        Total: {total}/6 {total !== 6 && "— un god clone necesita 6 genes"}
      </p>
    </div>
  );
}
