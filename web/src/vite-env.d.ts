/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WEBHOOK_PETICIONES?: string;
  readonly VITE_WEBHOOK_SUGERENCIAS?: string;
  readonly VITE_WEBHOOK_REPORTES?: string;
  readonly VITE_WEBHOOK_CONTACTO?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
