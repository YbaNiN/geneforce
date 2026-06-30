// Cliente para enviar formularios al Worker de Cloudflare (que reenvía a Discord).
//
// La URL del Worker se configura en una variable de entorno de Vite
// (VITE_FORMS_ENDPOINT). En su defecto usa una cadena vacía, lo que hará que
// el envío falle de forma controlada con un mensaje claro.

export type FormKind = "peticiones" | "sugerencias" | "reportes" | "contacto";

export interface FieldDef {
  name: string;
  label: string;
  type: "text" | "textarea" | "email" | "select";
  required: boolean;
  placeholder?: string;
  options?: string[]; // para type "select"
  maxLength: number;
}

export interface FormDef {
  kind: FormKind;
  label: string;
  title: string;
  description: string;
  fields: FieldDef[];
}

// Debe coincidir con la validación del Worker (worker/src/index.js).
export const FORM_DEFS: Record<FormKind, FormDef> = {
  peticiones: {
    kind: "peticiones",
    label: "Peticiones",
    title: "Enviar una petición",
    description: "¿Quieres pedir una función o cambio? Cuéntanoslo.",
    fields: [
      {
        name: "usuario",
        label: "Usuario",
        type: "text",
        required: false,
        placeholder: "Tu usuario de Discord o nombre",
        maxLength: 80,
      },
      {
        name: "tipo",
        label: "Tipo",
        type: "select",
        required: false,
        options: ["Funcionalidad", "Mejora", "Otro"],
        maxLength: 40,
      },
      {
        name: "descripcion",
        label: "Descripción",
        type: "textarea",
        required: true,
        placeholder: "Describe tu petición con detalle",
        maxLength: 1800,
      },
    ],
  },
  sugerencias: {
    kind: "sugerencias",
    label: "Sugerencias",
    title: "Enviar una sugerencia",
    description: "¿Tienes una idea para mejorar GeneForge?",
    fields: [
      {
        name: "usuario",
        label: "Usuario",
        type: "text",
        required: false,
        placeholder: "Tu usuario de Discord o nombre",
        maxLength: 80,
      },
      {
        name: "sugerencia",
        label: "Sugerencia",
        type: "textarea",
        required: true,
        placeholder: "Cuéntanos tu idea",
        maxLength: 1800,
      },
    ],
  },
  reportes: {
    kind: "reportes",
    label: "Reportes",
    title: "Reportar un problema",
    description: "¿Algo no funciona? Ayúdanos a arreglarlo.",
    fields: [
      {
        name: "usuario",
        label: "Usuario",
        type: "text",
        required: false,
        placeholder: "Tu usuario de Discord o nombre",
        maxLength: 80,
      },
      {
        name: "problema",
        label: "¿Qué falla?",
        type: "text",
        required: true,
        placeholder: "Resumen del problema",
        maxLength: 300,
      },
      {
        name: "pasos",
        label: "Pasos para reproducirlo",
        type: "textarea",
        required: true,
        placeholder: "1. ...\n2. ...\n3. ...",
        maxLength: 1500,
      },
    ],
  },
  contacto: {
    kind: "contacto",
    label: "Contacto",
    title: "Contactar",
    description: "Escríbenos directamente.",
    fields: [
      {
        name: "nombre",
        label: "Nombre",
        type: "text",
        required: true,
        placeholder: "Tu nombre",
        maxLength: 80,
      },
      {
        name: "email",
        label: "Email",
        type: "email",
        required: true,
        placeholder: "tu@email.com",
        maxLength: 120,
      },
      {
        name: "mensaje",
        label: "Mensaje",
        type: "textarea",
        required: true,
        placeholder: "Tu mensaje",
        maxLength: 1800,
      },
    ],
  },
};

// Webhooks de Discord, inyectados en build-time desde variables de entorno de
// GitHub Actions (no quedan escritos en el código fuente del repo).
const WEBHOOKS: Record<FormKind, string | undefined> = {
  peticiones: import.meta.env.VITE_WEBHOOK_PETICIONES,
  sugerencias: import.meta.env.VITE_WEBHOOK_SUGERENCIAS,
  reportes: import.meta.env.VITE_WEBHOOK_REPORTES,
  contacto: import.meta.env.VITE_WEBHOOK_CONTACTO,
};

// Metadatos de presentación del mensaje en Discord (título y color del embed).
const EMBED_META: Record<FormKind, { title: string; color: number }> = {
  peticiones: { title: "📋 Nueva petición", color: 0xc1501f },
  sugerencias: { title: "💡 Nueva sugerencia", color: 0xd4c84a },
  reportes: { title: "🐛 Nuevo reporte", color: 0xb54a3a },
  contacto: { title: "✉️ Nuevo contacto", color: 0x6bb8a8 },
};

export interface SubmitResult {
  ok: boolean;
  error?: string;
}

/**
 * Envía un formulario directamente al webhook de Discord correspondiente.
 * Construye un embed con los campos no vacíos del formulario.
 */
export async function submitForm(
  kind: FormKind,
  fields: Record<string, string>,
): Promise<SubmitResult> {
  const webhookUrl = WEBHOOKS[kind];
  if (!webhookUrl) {
    return {
      ok: false,
      error: "Este formulario no está configurado todavía.",
    };
  }

  const def = FORM_DEFS[kind];
  const meta = EMBED_META[kind];

  // Construye los campos del embed a partir de las definiciones del formulario,
  // respetando el orden y omitiendo los vacíos.
  const embedFields = def.fields
    .map((f) => ({ name: f.label, value: (fields[f.name] ?? "").trim() }))
    .filter((f) => f.value.length > 0)
    .map((f) => ({
      name: f.name,
      value: f.value.slice(0, 1024),
      inline: false,
    }));

  const payload = {
    embeds: [
      {
        title: meta.title,
        color: meta.color,
        fields: embedFields,
        timestamp: new Date().toISOString(),
        footer: { text: "GeneForge" },
      },
    ],
  };

  try {
    const resp = await fetch(webhookUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    // Discord responde 204 No Content cuando el webhook se envía bien.
    if (!resp.ok) {
      return { ok: false, error: "No se pudo enviar el mensaje" };
    }
    return { ok: true };
  } catch {
    return { ok: false, error: "No se pudo conectar con Discord" };
  }
}
