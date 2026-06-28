//! Representación de una planta: 6 slots de gen.

use crate::gene::Gene;
use std::collections::HashMap;
use std::fmt;

pub const SLOT_COUNT: usize = 6;

/// Una planta o clon, con sus 6 genes en orden de slot.
/// El orden importa para el cálculo de cruce (es por columna),
/// pero no importa para comparar contra un objetivo (ver `gene_counts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Plant {
    pub slots: [Gene; SLOT_COUNT],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePlantError {
    WrongLength(usize),
    InvalidChar(char),
}

impl fmt::Display for ParsePlantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsePlantError::WrongLength(n) => {
                write!(f, "se esperaban {} genes, se recibieron {}", SLOT_COUNT, n)
            }
            ParsePlantError::InvalidChar(c) => write!(f, "carácter de gen inválido: '{}'", c),
        }
    }
}

impl Plant {
    pub fn new(slots: [Gene; SLOT_COUNT]) -> Self {
        Plant { slots }
    }

    /// Parsea una cadena de 6 caracteres (ej. "GGGYYH") a una Plant.
    pub fn parse(s: &str) -> Result<Plant, ParsePlantError> {
        let chars: Vec<char> = s.trim().chars().collect();
        if chars.len() != SLOT_COUNT {
            return Err(ParsePlantError::WrongLength(chars.len()));
        }
        let mut slots = [Gene::X; SLOT_COUNT];
        for (i, c) in chars.iter().enumerate() {
            slots[i] = Gene::from_char(*c).ok_or(ParsePlantError::InvalidChar(*c))?;
        }
        Ok(Plant::new(slots))
    }

    /// Cuenta cuántas veces aparece cada tipo de gen, sin importar la posición.
    /// Usado para comparar contra un objetivo tipo "3G 3Y" donde el orden no importa.
    pub fn gene_counts(&self) -> HashMap<Gene, u8> {
        let mut counts = HashMap::new();
        for g in self.slots {
            *counts.entry(g).or_insert(0) += 1;
        }
        counts
    }

    /// Número de genes rojos (X o W) en la planta. Útil para scoring/penalización.
    pub fn red_count(&self) -> u8 {
        self.slots.iter().filter(|g| g.is_red()).count() as u8
    }

    pub fn green_count(&self) -> u8 {
        self.slots.iter().filter(|g| g.is_green()).count() as u8
    }
}

impl fmt::Display for Plant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for g in self.slots {
            write!(f, "{}", g)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gene::Gene::*;

    #[test]
    fn parses_valid_string() {
        let p = Plant::parse("GGGYYH").unwrap();
        assert_eq!(p.slots, [G, G, G, Y, Y, H]);
    }

    #[test]
    fn parses_lowercase() {
        let p = Plant::parse("gggyyh").unwrap();
        assert_eq!(p.slots, [G, G, G, Y, Y, H]);
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(Plant::parse("GGG"), Err(ParsePlantError::WrongLength(3)));
        assert_eq!(
            Plant::parse("GGGYYHH"),
            Err(ParsePlantError::WrongLength(7))
        );
    }

    #[test]
    fn rejects_invalid_char() {
        assert_eq!(Plant::parse("GGGYYZ"), Err(ParsePlantError::InvalidChar('Z')));
    }

    #[test]
    fn gene_counts_ignores_order() {
        let a = Plant::parse("GGGYYH").unwrap();
        let b = Plant::parse("GYGYGH").unwrap();
        assert_eq!(a.gene_counts(), b.gene_counts());
    }

    #[test]
    fn red_and_green_counts() {
        let p = Plant::parse("GGGYYX").unwrap();
        assert_eq!(p.green_count(), 5);
        assert_eq!(p.red_count(), 1);
    }

    #[test]
    fn display_roundtrips() {
        let p = Plant::parse("GYHXWG").unwrap();
        assert_eq!(p.to_string(), "GYHXWG");
    }
}
