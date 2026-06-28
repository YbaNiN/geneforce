/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Superficies: del más oscuro (fondo) al más claro (paneles elevados).
        carbon: {
          900: "#0f0e0b",
          800: "#15130f",
          700: "#1c1914",
          600: "#252119",
          500: "#322c22",
        },
        // Acento óxido — la identidad visual del juego.
        rust: {
          DEFAULT: "#c1501f",
          light: "#e0843f",
          dark: "#8a3815",
        },
        // Bordes/metal.
        steel: {
          DEFAULT: "#3a342a",
          light: "#4a443a",
          muted: "#5a5448",
        },
        // Texto.
        bone: {
          DEFAULT: "#e8e3d8",
          muted: "#8a8276",
          dim: "#5f5a50",
        },
        // Colores de gen (semánticos, no literales del juego, para legibilidad).
        gene: {
          g: "#7fb86b", // growth - verde
          y: "#d4c84a", // yield - ámbar
          h: "#6bb8a8", // hardiness - teal
          x: "#8a7060", // blank - marrón apagado
          w: "#b54a3a", // water - rojo ladrillo
        },
      },
      fontFamily: {
        display: ['"Oswald"', "sans-serif"],
        mono: ['"JetBrains Mono"', "monospace"],
      },
      borderRadius: {
        DEFAULT: "2px",
      },
    },
  },
  plugins: [],
};
