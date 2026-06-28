//! Lógica de crossbreeding: centro vs. donantes, por slot.
//!
//! Regla validada (ver SPEC_VALIDADA.md, sección 2):
//! - Por cada slot, se agrupan los pesos de los donantes POR TIPO EXACTO de gen.
//! - Un tipo de gen donante solo sobrescribe al centro si su peso combinado
//!   es ESTRICTAMENTE MAYOR que el peso base del gen del centro.
//! - Si ningún tipo supera al centro, el centro se conserva.
//! - Si varios tipos empatan en el máximo (y todos superan al centro),
//!   el resultado es aleatorio uniforme entre ellos.

use crate::gene::Gene;
use crate::plant::{Plant, SLOT_COUNT};
use rand::Rng;
use std::collections::BTreeMap;

/// Resultado del cálculo de un slot: qué genes son posibles y con qué probabilidad.
/// Si solo hay una entrada con probabilidad 1.0, el resultado es determinista.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotOutcome {
    /// Mapeo gen -> probabilidad (suma 1.0). BTreeMap para orden estable en tests/UI.
    pub probabilities: BTreeMap<Gene, f64>,
}

impl SlotOutcome {
    fn certain(gene: Gene) -> Self {
        let mut probabilities = BTreeMap::new();
        probabilities.insert(gene, 1.0);
        SlotOutcome { probabilities }
    }

    /// true si el resultado es 100% seguro (sin azar involucrado).
    pub fn is_certain(&self) -> bool {
        self.probabilities.len() == 1
    }

    /// Probabilidad de obtener un gen específico (0.0 si no es posible).
    pub fn probability_of(&self, gene: Gene) -> f64 {
        *self.probabilities.get(&gene).unwrap_or(&0.0)
    }

    /// Elige un resultado al azar respetando las probabilidades (para simulación/UI).
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Gene {
        if self.probabilities.len() == 1 {
            return *self.probabilities.keys().next().unwrap();
        }
        let roll: f64 = rng.gen_range(0.0..1.0);
        let mut acc = 0.0;
        for (gene, p) in &self.probabilities {
            acc += p;
            if roll < acc {
                return *gene;
            }
        }
        // Fallback por redondeo de floats: devuelve el último.
        *self.probabilities.keys().last().unwrap()
    }
}

/// Calcula el resultado de un slot dado el gen del centro y los genes donantes
/// (vecinos) en esa misma posición.
pub fn slot_outcome(center: Gene, donors: &[Gene]) -> SlotOutcome {
    if donors.is_empty() {
        return SlotOutcome::certain(center);
    }

    let mut donor_weight: BTreeMap<Gene, f64> = BTreeMap::new();
    for &g in donors {
        *donor_weight.entry(g).or_insert(0.0) += g.weight();
    }

    let center_weight = center.weight();

    // Solo los tipos que superan ESTRICTAMENTE al centro son candidatos ("challengers").
    let challengers: BTreeMap<Gene, f64> = donor_weight
        .into_iter()
        .filter(|(_, w)| *w > center_weight)
        .collect();

    if challengers.is_empty() {
        return SlotOutcome::certain(center);
    }

    let max_w = challengers
        .values()
        .cloned()
        .fold(f64::MIN, f64::max);

    // Tolerancia para comparar floats (evita falsos empates/no-empates por redondeo).
    const EPS: f64 = 1e-9;
    let winners: Vec<Gene> = challengers
        .iter()
        .filter(|(_, w)| (**w - max_w).abs() < EPS)
        .map(|(g, _)| *g)
        .collect();

    if winners.len() == 1 {
        return SlotOutcome::certain(winners[0]);
    }

    let p = 1.0 / winners.len() as f64;
    let mut probabilities = BTreeMap::new();
    for g in winners {
        probabilities.insert(g, p);
    }
    SlotOutcome { probabilities }
}

/// Resultado completo de cruzar un centro con un conjunto de donantes:
/// un SlotOutcome por cada uno de los 6 slots.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbreedResult {
    pub outcomes: [SlotOutcome; SLOT_COUNT],
}

impl CrossbreedResult {
    /// Probabilidad de que el cruce produzca EXACTAMENTE la planta `target`
    /// en este orden de slots (probabilidades de cada slot multiplicadas).
    pub fn probability_of_exact(&self, target: &Plant) -> f64 {
        self.outcomes
            .iter()
            .zip(target.slots.iter())
            .map(|(outcome, gene)| outcome.probability_of(*gene))
            .product()
    }

    /// true si todos los slots son deterministas (sin ningún azar).
    pub fn is_fully_deterministic(&self) -> bool {
        self.outcomes.iter().all(|o| o.is_certain())
    }

    /// Si el resultado es 100% determinista, devuelve la `Plant` concreta que
    /// produce. Devuelve `None` si algún slot tiene azar (empate sin resolver).
    /// Usado para fabricar clones intermedios ("bridges") en rutas GEN.2.
    pub fn as_plant(&self) -> Option<Plant> {
        if !self.is_fully_deterministic() {
            return None;
        }
        let mut slots = [crate::gene::Gene::X; SLOT_COUNT];
        for (i, outcome) in self.outcomes.iter().enumerate() {
            // is_certain garantiza exactamente una entrada en probabilities.
            slots[i] = *outcome.probabilities.keys().next().unwrap();
        }
        Some(Plant::new(slots))
    }
}

/// Cruza una planta central con un conjunto de plantas donantes (vecinas).
pub fn crossbreed(center: &Plant, donors: &[Plant]) -> CrossbreedResult {
    let outcomes: [SlotOutcome; SLOT_COUNT] = std::array::from_fn(|slot_idx| {
        let donor_genes: Vec<Gene> = donors.iter().map(|d| d.slots[slot_idx]).collect();
        slot_outcome(center.slots[slot_idx], &donor_genes)
    });

    CrossbreedResult { outcomes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gene::Gene::*;

    // --- Tests directamente portados de la validación en Python (crossbreed_v2.py) ---

    #[test]
    fn donors_below_center_weight_dont_challenge() {
        // center=X (1.0), donors=[Y, G] (0.6 cada uno, no superan 1.0 individualmente)
        let outcome = slot_outcome(X, &[Y, G]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(X), 1.0);
    }

    #[test]
    fn tie_with_center_keeps_center() {
        // center=H (0.6), donors=[Y, G] (0.6 cada uno, empatan con el centro, NO superan)
        let outcome = slot_outcome(H, &[Y, G]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(H), 1.0);
    }

    #[test]
    fn two_greens_beat_one_red_center() {
        // center=X (1.0), donors=[Y, Y] (1.2 > 1.0) -> ganan Y
        let outcome = slot_outcome(X, &[Y, Y]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(Y), 1.0);
    }

    #[test]
    fn single_red_donor_beats_green_center() {
        // center=Y (0.6), donors=[X] (1.0 > 0.6) -> gana X
        let outcome = slot_outcome(Y, &[X]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(X), 1.0);
    }

    #[test]
    fn separate_red_types_dont_combine() {
        // center=X (1.0), donors=[G, W]: G(0.6) no alcanza, W(1.0) solo iguala -> no supera
        let outcome = slot_outcome(X, &[G, W]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(X), 1.0);
    }

    #[test]
    fn three_greens_beat_red_and_extra_red_center() {
        // center=W (1.0), donors=[G,G,G,X]: G suma 1.8 > 1.0 -> gana G
        let outcome = slot_outcome(W, &[G, G, G, X]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(G), 1.0);
    }

    #[test]
    fn no_donors_keeps_center() {
        let outcome = slot_outcome(Y, &[]);
        assert!(outcome.is_certain());
        assert_eq!(outcome.probability_of(Y), 1.0);
    }

    #[test]
    fn classic_tie_case_from_rustgenetics_source() {
        // Caso citado textualmente: center=X(1.0), donors=[G,G,Y,Y] -> G=1.2, Y=1.2, empate 50/50
        let outcome = slot_outcome(X, &[G, G, Y, Y]);
        assert!(!outcome.is_certain());
        assert_eq!(outcome.probability_of(G), 0.5);
        assert_eq!(outcome.probability_of(Y), 0.5);
        assert_eq!(outcome.probability_of(X), 0.0);
    }

    #[test]
    fn three_way_tie_splits_evenly() {
        // center=X(1.0). Si G, Y, H cada uno aporta 1.2 (dos donantes cada tipo), triple empate.
        let outcome = slot_outcome(X, &[G, G, Y, Y, H, H]);
        assert!(!outcome.is_certain());
        let p = 1.0 / 3.0;
        assert!((outcome.probability_of(G) - p).abs() < 1e-9);
        assert!((outcome.probability_of(Y) - p).abs() < 1e-9);
        assert!((outcome.probability_of(H) - p).abs() < 1e-9);
    }

    #[test]
    fn full_crossbreed_combines_all_six_slots() {
        let center = Plant::parse("XXXXXX").unwrap();
        let donor = Plant::parse("GGGGGG").unwrap();
        let result = crossbreed(&center, &[donor, donor]); // dos donantes con todo G
        assert!(result.is_fully_deterministic());
        let target = Plant::parse("GGGGGG").unwrap();
        assert_eq!(result.probability_of_exact(&target), 1.0);
    }

    #[test]
    fn probability_of_exact_multiplies_across_independent_ties() {
        // Construimos 2 slots con empate 50/50 cada uno, independientes entre sí,
        // y confirmamos que la probabilidad de un resultado exacto es 0.5 * 0.5 = 0.25.
        //
        // center = X en todos los slots relevantes.
        // slot0: donors [G, G, Y, Y] -> G=1.2, Y=1.2, ambos > 1.0 -> tie 50/50.
        // slot1: igual patrón -> otro tie 50/50 independiente.
        let center = Plant::parse("XXXXXX").unwrap();
        let donor1 = Plant::parse("GGXXXX").unwrap(); // aporta G a slot0 y slot1
        let donor2 = Plant::parse("GGXXXX").unwrap(); // refuerza G (suma 1.2 en cada slot)
        let donor3 = Plant::parse("YYXXXX").unwrap(); // aporta Y a slot0 y slot1
        let donor4 = Plant::parse("YYXXXX").unwrap(); // refuerza Y (suma 1.2 en cada slot)

        let result = crossbreed(&center, &[donor1, donor2, donor3, donor4]);

        assert!(!result.outcomes[0].is_certain());
        assert!(!result.outcomes[1].is_certain());
        assert_eq!(result.outcomes[0].probability_of(Gene::G), 0.5);
        assert_eq!(result.outcomes[1].probability_of(Gene::G), 0.5);

        // Target: GG en slots 0 y 1, resto X (que se mantiene determinista por defecto).
        let target = Plant::parse("GGXXXX").unwrap();
        let prob = result.probability_of_exact(&target);
        assert!((prob - 0.25).abs() < 1e-9);
    }

    #[test]
    fn as_plant_returns_concrete_plant_when_deterministic() {
        let center = Plant::parse("XXXXXX").unwrap();
        let donor = Plant::parse("GGGYYY").unwrap();
        let result = crossbreed(&center, &[donor, donor]);
        let plant = result.as_plant().expect("debería ser determinista");
        assert_eq!(plant, Plant::parse("GGGYYY").unwrap());
    }

    #[test]
    fn as_plant_returns_none_when_uncertain() {
        let center = Plant::parse("XXXXXX").unwrap();
        // slot0 con empate G/Y -> no determinista
        let result = crossbreed(
            &center,
            &[
                Plant::parse("GXXXXX").unwrap(),
                Plant::parse("GXXXXX").unwrap(),
                Plant::parse("YXXXXX").unwrap(),
                Plant::parse("YXXXXX").unwrap(),
            ],
        );
        assert!(result.as_plant().is_none());
    }
}
