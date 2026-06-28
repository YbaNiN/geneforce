//! Tipos de gen y sus propiedades fundamentales.
//!
//! Reglas validadas (ver SPEC_VALIDADA.md):
//! - 5 tipos de gen: G, Y, H (verdes/positivos) y X, W (rojos/negativos).
//! - Peso: verde = 0.6, rojo = 1.0 (confirmado en 3 fuentes independientes).

use std::fmt;

/// Un gen individual. Cada planta tiene exactamente 6, uno por slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gene {
    /// Growth — aumenta la velocidad de crecimiento. Verde.
    G,
    /// Yield — aumenta el rendimiento de cosecha. Verde.
    Y,
    /// Hardiness — resistencia ambiental (frío/calor). Verde.
    H,
    /// Blank — slot sin efecto. Rojo.
    X,
    /// Water — alto consumo de agua. Rojo.
    W,
}

/// Color/categoría de un gen, usado para presentación en la UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneColor {
    Green,
    Red,
}

impl Gene {
    /// Las 5 variantes, en un orden estable (útil para iterar en la UI/solver).
    pub const ALL: [Gene; 5] = [Gene::G, Gene::Y, Gene::H, Gene::X, Gene::W];

    /// Peso base del gen, usado en la competencia centro-vs-donantes.
    /// Verde = 0.6, rojo = 1.0. Confirmado contra múltiples fuentes del juego.
    pub fn weight(self) -> f64 {
        match self {
            Gene::G | Gene::Y | Gene::H => 0.6,
            Gene::X | Gene::W => 1.0,
        }
    }

    pub fn color(self) -> GeneColor {
        match self {
            Gene::G | Gene::Y | Gene::H => GeneColor::Green,
            Gene::X | Gene::W => GeneColor::Red,
        }
    }

    pub fn is_green(self) -> bool {
        matches!(self.color(), GeneColor::Green)
    }

    pub fn is_red(self) -> bool {
        matches!(self.color(), GeneColor::Red)
    }

    /// Parsea un único carácter ASCII a un Gene. Mayúsculas o minúsculas.
    pub fn from_char(c: char) -> Option<Gene> {
        match c.to_ascii_uppercase() {
            'G' => Some(Gene::G),
            'Y' => Some(Gene::Y),
            'H' => Some(Gene::H),
            'X' => Some(Gene::X),
            'W' => Some(Gene::W),
            _ => None,
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Gene::G => 'G',
            Gene::Y => 'Y',
            Gene::H => 'H',
            Gene::X => 'X',
            Gene::W => 'W',
        }
    }
}

impl fmt::Display for Gene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_match_validated_spec() {
        assert_eq!(Gene::G.weight(), 0.6);
        assert_eq!(Gene::Y.weight(), 0.6);
        assert_eq!(Gene::H.weight(), 0.6);
        assert_eq!(Gene::X.weight(), 1.0);
        assert_eq!(Gene::W.weight(), 1.0);
    }

    #[test]
    fn colors_are_correct() {
        assert!(Gene::G.is_green());
        assert!(Gene::Y.is_green());
        assert!(Gene::H.is_green());
        assert!(Gene::X.is_red());
        assert!(Gene::W.is_red());
    }

    #[test]
    fn parses_chars_case_insensitively() {
        assert_eq!(Gene::from_char('g'), Some(Gene::G));
        assert_eq!(Gene::from_char('G'), Some(Gene::G));
        assert_eq!(Gene::from_char('w'), Some(Gene::W));
        assert_eq!(Gene::from_char('Z'), None);
    }

    #[test]
    fn display_roundtrips_with_from_char() {
        for g in Gene::ALL {
            let s = g.to_string();
            let parsed = Gene::from_char(s.chars().next().unwrap());
            assert_eq!(parsed, Some(g));
        }
    }
}
