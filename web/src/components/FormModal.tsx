import { useState, useEffect, useCallback } from "react";
import {
  FORM_DEFS,
  submitForm,
  type FormKind,
} from "../lib/forms";

interface FormModalProps {
  kind: FormKind | null;
  onClose: () => void;
}

type Status = "idle" | "sending" | "sent" | "error";

export function FormModal({ kind, onClose }: FormModalProps) {
  const [values, setValues] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<Status>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Reinicia el estado cada vez que se abre un formulario distinto.
  useEffect(() => {
    setValues({});
    setStatus("idle");
    setErrorMsg(null);
  }, [kind]);

  // Cerrar con tecla Escape.
  const handleKey = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    },
    [onClose],
  );
  useEffect(() => {
    if (kind) {
      document.addEventListener("keydown", handleKey);
      return () => document.removeEventListener("keydown", handleKey);
    }
  }, [kind, handleKey]);

  if (!kind) return null;
  const def = FORM_DEFS[kind];

  function setField(name: string, value: string) {
    setValues((prev) => ({ ...prev, [name]: value }));
  }

  function validateLocally(): string | null {
    for (const f of def.fields) {
      const v = (values[f.name] ?? "").trim();
      if (f.required && !v) return `Falta el campo: ${f.label}`;
      if (f.type === "email" && v && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v)) {
        return "El email no es válido";
      }
    }
    return null;
  }

  async function handleSubmit() {
    const localError = validateLocally();
    if (localError) {
      setErrorMsg(localError);
      setStatus("error");
      return;
    }
    setStatus("sending");
    setErrorMsg(null);

    const result = await submitForm(kind!, values);
    if (result.ok) {
      setStatus("sent");
    } else {
      setErrorMsg(result.error ?? "Error al enviar");
      setStatus("error");
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="form-modal-title"
    >
      <div
        className="panel bg-carbon-700 w-full max-w-md max-h-[90vh] overflow-y-auto p-5"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between mb-1">
          <h2
            id="form-modal-title"
            className="text-lg font-bold uppercase tracking-wide text-bone"
          >
            {def.title}
          </h2>
          <button
            onClick={onClose}
            className="text-bone-dim hover:text-bone text-xl leading-none -mt-1"
            aria-label="Cerrar"
          >
            ✕
          </button>
        </div>
        <p className="text-bone-muted text-xs mb-4">{def.description}</p>

        {status === "sent" ? (
          <div className="text-center py-6">
            <div className="text-gene-g text-3xl mb-2">✓</div>
            <p className="text-bone text-sm">¡Enviado! Gracias por tu mensaje.</p>
            <button
              onClick={onClose}
              className="btn-rust px-5 py-2 text-xs rounded-sm mt-4"
            >
              Cerrar
            </button>
          </div>
        ) : (
          <div className="space-y-3">
            {def.fields.map((f) => (
              <div key={f.name}>
                <label className="text-xs uppercase tracking-wide text-bone-muted block mb-1">
                  {f.label}
                  {f.required && <span className="text-rust"> *</span>}
                </label>

                {f.type === "textarea" ? (
                  <textarea
                    value={values[f.name] ?? ""}
                    onChange={(e) => setField(f.name, e.target.value)}
                    placeholder={f.placeholder}
                    maxLength={f.maxLength}
                    rows={4}
                    className="w-full bg-carbon-800 border border-steel rounded-sm px-3 py-2 text-sm text-bone placeholder:text-bone-dim focus:outline-none focus:border-rust transition-colors resize-y"
                  />
                ) : f.type === "select" ? (
                  <select
                    value={values[f.name] ?? ""}
                    onChange={(e) => setField(f.name, e.target.value)}
                    className="w-full bg-carbon-800 border border-steel rounded-sm px-3 py-2 text-sm text-bone focus:outline-none focus:border-rust transition-colors"
                  >
                    <option value="">— elige —</option>
                    {f.options?.map((opt) => (
                      <option key={opt} value={opt}>
                        {opt}
                      </option>
                    ))}
                  </select>
                ) : (
                  <input
                    type={f.type}
                    value={values[f.name] ?? ""}
                    onChange={(e) => setField(f.name, e.target.value)}
                    placeholder={f.placeholder}
                    maxLength={f.maxLength}
                    className="w-full bg-carbon-800 border border-steel rounded-sm px-3 py-2 text-sm text-bone placeholder:text-bone-dim focus:outline-none focus:border-rust transition-colors"
                  />
                )}
              </div>
            ))}

            {errorMsg && (
              <p className="text-gene-w text-xs font-mono">{errorMsg}</p>
            )}

            <button
              onClick={handleSubmit}
              disabled={status === "sending"}
              className="btn-rust w-full py-2.5 text-sm rounded-sm mt-1"
            >
              {status === "sending" ? "Enviando…" : "Enviar"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
