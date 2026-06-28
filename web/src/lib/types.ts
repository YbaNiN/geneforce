// Tipos del dominio, espejando las estructuras del núcleo Rust.

export type GeneChar = "G" | "Y" | "H" | "X" | "W";

export const GENES: GeneChar[] = ["G", "Y", "H", "X", "W"];
export const GREEN_GENES: GeneChar[] = ["G", "Y", "H"];
export const RED_GENES: GeneChar[] = ["X", "W"];

export const GENE_LABELS: Record<GeneChar, string> = {
  G: "Growth",
  Y: "Yield",
  H: "Hardiness",
  X: "Empty",
  W: "Water",
};

export const GENE_DESCRIPTIONS: Record<GeneChar, string> = {
  G: "Aumenta la velocidad de crecimiento",
  Y: "Aumenta el rendimiento de cosecha",
  H: "Resistencia a temperaturas extremas",
  X: "Slot vacío, sin efecto",
  W: "Aumenta el consumo de agua",
};

export function isGreen(g: GeneChar): boolean {
  return GREEN_GENES.includes(g);
}

export interface GeneProb {
  gene: GeneChar;
  probability: number;
}

export interface SlotResult {
  options: GeneProb[];
  certain: boolean;
}

export interface CrossResult {
  slots: SlotResult[];
  fully_deterministic: boolean;
}

export interface CandidateOut {
  center: string;
  donors: string[];
  result: CrossResult;
  success_probability: number;
  expected_red_genes: number;
  donor_count: number;
}

export interface InvalidPlant {
  input: string;
  error: string;
}

export interface TwoStepOut {
  step1: CandidateOut;
  bridge: string;
  step2: CandidateOut;
  total_probability: number;
}

export interface SolveResponse {
  candidates: CandidateOut[];
  two_step: TwoStepOut[];
  invalid: InvalidPlant[];
}

export interface TargetSpec {
  counts: [string, number][];
}

export interface SolveRequest {
  pool: string[];
  target: TargetSpec;
  max_donors?: number;
  min_donors?: number;
  top_n?: number;
  allow_two_gen?: boolean;
}

// Presets de objetivo comunes ("god clones"), confirmados en las guías del juego.
export interface TargetPreset {
  id: string;
  label: string;
  description: string;
  counts: Partial<Record<GeneChar, number>>;
}

export const TARGET_PRESETS: TargetPreset[] = [
  {
    id: "3g3y",
    label: "3G 3Y",
    description: "El estándar — mejor rendimiento por hora",
    counts: { G: 3, Y: 3 },
  },
  {
    id: "4y2g",
    label: "4Y 2G",
    description: "Máximo yield, menos clonado frecuente",
    counts: { G: 2, Y: 4 },
  },
  {
    id: "4g2y",
    label: "4G 2Y",
    description: "Build de velocidad de crecimiento",
    counts: { G: 4, Y: 2 },
  },
];
