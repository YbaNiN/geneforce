interface OptionsPanelProps {
  maxDonors: number;
  onMaxDonorsChange: (n: number) => void;
  allowTwoGen: boolean;
  onAllowTwoGenChange: (v: boolean) => void;
}

export function OptionsPanel({
  maxDonors,
  onMaxDonorsChange,
  allowTwoGen,
  onAllowTwoGenChange,
}: OptionsPanelProps) {
  return (
    <div className="panel p-3.5">
      <div className="mb-3.5">
        <div className="flex justify-between text-xs mb-1.5">
          <span className="text-bone">Plantas circundantes</span>
          <span className="text-rust-light font-mono">{maxDonors}</span>
        </div>
        <input
          type="range"
          min={1}
          max={8}
          step={1}
          value={maxDonors}
          onChange={(e) => onMaxDonorsChange(parseInt(e.target.value))}
          className="w-full accent-rust"
        />
        <p className="text-[11px] text-bone-dim mt-1">
          Más circundantes = más combinaciones, pero cálculo más lento.
        </p>
      </div>

      <label className="flex items-center justify-between cursor-pointer">
        <span className="text-xs text-bone">
          Permitir 2 generaciones (GEN.2)
        </span>
        <button
          role="switch"
          aria-checked={allowTwoGen}
          onClick={() => onAllowTwoGenChange(!allowTwoGen)}
          className={`w-9 h-[18px] rounded-full relative transition-colors ${
            allowTwoGen ? "bg-rust" : "bg-steel"
          }`}
        >
          <span
            className={`w-3.5 h-3.5 bg-carbon-900 rounded-full absolute top-0.5 transition-all ${
              allowTwoGen ? "right-0.5" : "left-0.5"
            }`}
          />
        </button>
      </label>
    </div>
  );
}
