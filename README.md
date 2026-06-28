# 🧬 GeneForge

**Calculadora de cruce genético para el videojuego [Rust](https://rust.facepunch.com/).**
Encuentra las mejores combinaciones de clones para forjar un *god clone* (plantas
con genética óptima como `GGGYYY`) para hemp, berries y otros cultivos.

El motor de cálculo está escrito en **Rust** y compilado a **WebAssembly**, con una
interfaz **React + TypeScript** de estética industrial. Todo corre en el navegador:
sin servidor, sin cuentas, sin latencia.

![status](https://img.shields.io/badge/tests-32%20passing-success)
![rust](https://img.shields.io/badge/core-Rust%20%2B%20WASM-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

> **Proyecto no oficial, con fines educativos.** GeneForge no está afiliado a
> Facepunch Studios ni a ningún sitio de cálculo existente. Reimplementa de forma
> independiente la mecánica de cruce de plantas del juego, documentada por la
> comunidad, como ejercicio de ingeniería (Rust → WASM → React).

---

## ✨ Características

- **Solver de un paso (GEN.1):** prueba todas las combinaciones de centro +
  circundantes de tu pool y las rankea por probabilidad de lograr el objetivo.
- **Rutas de dos pasos (GEN.2):** cuando el objetivo no es alcanzable en una
  ronda, fabrica un clon intermedio determinista y lo usa en una segunda ronda.
- **Cálculo exacto de probabilidad:** los empates entre genes se modelan como
  "slots en disputa" y las probabilidades se combinan correctamente.
- **100% en el navegador:** motor Rust compilado a WebAssembly. Hosting estático.
- **Estética fiel:** tema oscuro, óxido e industrial, en línea con el juego.

## 🏗️ Arquitectura

```
geneforge/
├── core/          # Núcleo de dominio en Rust (lógica pura, sin web)
│   └── src/
│       ├── gene.rs        # Tipos de gen y pesos (G/Y/H=0.6, X/W=1.0)
│       ├── plant.rs       # Planta de 6 slots, parsing, conteos
│       ├── crossbreed.rs  # Lógica centro-vs-donantes + empates
│       └── solver.rs      # Búsqueda de combinaciones (GEN.1 y GEN.2)
├── wasm/          # Bindings WASM (wasm-bindgen) que envuelven el núcleo
│   └── src/lib.rs
└── web/           # Frontend React + Vite + Tailwind
    └── src/
        ├── lib/           # Wrapper TS del WASM y tipos de dominio
        ├── components/    # Componentes de UI
        └── App.tsx
```

### ¿Por qué Rust → WASM en el frontend?

El cálculo es puro y determinista (combinatoria + reglas de comparación de genes):
no necesita backend ni base de datos. Compilarlo a WASM da cálculo instantáneo sin
latencia de red, hosting estático trivial, y fidelidad al espíritu del proyecto —
Rust (el lenguaje) calculando la genética de Rust (el juego).

## 🚀 Cómo ejecutar

**Requisitos:** [Rust](https://rustup.rs/) + [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/), y Node.js 18+.

```bash
# 1. Añadir el target de WASM (una sola vez)
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# 2. Instalar dependencias del frontend
cd web
npm install

# 3. Compilar el núcleo Rust a WASM (genera web/src/wasm/)
npm run build:wasm

# 4. Arrancar el servidor de desarrollo
npm run dev
```

Build de producción:

```bash
npm run build:wasm   # imprescindible antes del build de la web
npm run build        # genera web/dist/, listo para hosting estático
```

El contenido de `web/dist/` se despliega tal cual en Vercel, Netlify, Cloudflare
Pages, GitHub Pages, etc.

## 🧪 Testing

```bash
cd core
cargo test       # 32 tests del núcleo
cargo clippy     # lint (sin warnings)
```

## 🔬 La mecánica de cruce

Cada planta tiene 6 slots. Tipos de gen: `G` (growth), `Y` (yield), `H` (hardiness)
son verdes (peso `0.6`); `X` (vacío) y `W` (agua) son rojos (peso `1.0`).

El cruce se resuelve **por slot, de forma independiente**, con un modelo de
**1 planta central (defensora) vs. N circundantes (donantes)**:

1. Por cada slot, se suma el peso de los donantes agrupado por tipo exacto de gen.
2. Un tipo donante sobrescribe al centro solo si su peso combinado es
   **estrictamente mayor** que el del gen central.
3. Si nadie supera al centro, este conserva su gen.
4. Si varios tipos empatan en el máximo (y superan al centro), el resultado es
   aleatorio uniforme entre ellos — un "slot en disputa".

Estas reglas están cubiertas por 32 tests unitarios, incluida una verificación
Monte Carlo que confirma que el cálculo analítico coincide con la simulación.

## 📋 Fuera de alcance

- Lectura automática de genes desde el proceso del juego (requiere permisos
  especiales fuera del alcance de una web).
- Persistencia entre sesiones (la calculadora es stateless por diseño).

## 📄 Licencia

MIT — ver [LICENSE](./LICENSE).
