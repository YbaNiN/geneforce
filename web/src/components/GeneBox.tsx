import type { GeneChar } from "../lib/types";

const GENE_CLASS: Record<GeneChar, string> = {
  G: "gene-g",
  Y: "gene-y",
  H: "gene-h",
  X: "gene-x",
  W: "gene-w",
};

interface GeneBoxProps {
  gene: GeneChar;
  size?: "sm" | "md" | "lg";
}

const SIZE_CLASS = {
  sm: "h-6 text-xs",
  md: "h-8 text-sm",
  lg: "h-10 text-base",
};

export function GeneBox({ gene, size = "md" }: GeneBoxProps) {
  return (
    <span
      className={`gene-box ${GENE_CLASS[gene]} ${SIZE_CLASS[size]}`}
      title={gene}
      aria-label={`Gen ${gene}`}
    >
      {gene}
    </span>
  );
}

interface TieBoxProps {
  genes: GeneChar[];
  size?: "sm" | "md" | "lg";
}

// Caja para un slot en disputa (empate): muestra los genes posibles con
// un patrón rayado que comunica visualmente la incertidumbre.
export function TieBox({ genes, size = "md" }: TieBoxProps) {
  const label = genes.join("/");
  return (
    <span
      className={`gene-box ${SIZE_CLASS[size]} text-carbon-900`}
      style={{
        background:
          "repeating-linear-gradient(45deg, #d4c84a 0 6px, #6bb8a8 6px 12px)",
      }}
      title={`En disputa: ${label}`}
      aria-label={`Slot en disputa entre ${label}`}
    >
      <span className="text-[10px] font-bold">{label}</span>
    </span>
  );
}
