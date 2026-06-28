//! Búsqueda de la mejor combinación de plantas (centro + donantes) para
//! alcanzar un objetivo de genética, dado un pool de plantas disponibles.
//!
//! Estrategia (ver SPEC_VALIDADA.md, secciones 5 y 7):
//! - Probar cada planta del pool como posible centro.
//! - Para cada centro, probar subconjuntos de las plantas restantes como donantes
//!   (tamaño 1 hasta `max_donors`, límite físico real del juego = 8).
//! - Para cada combinación, calcular el resultado de cruce y compararlo contra
//!   el objetivo (por conteo de genes, sin importar posición — confirmado en spec).
//! - Rankear por probabilidad de match exacto; en empate, penalizar genes rojos
//!   esperados y la cantidad de resultados posibles (criterio citado de jaretburkett,
//!   adaptado: aquí se usa como tie-breaker secundario, no como criterio único).

use crate::crossbreed::{crossbreed, CrossbreedResult};
use crate::gene::Gene;
use crate::plant::Plant;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::collections::HashMap;

/// Objetivo de cruce expresado como conteo de cada tipo de gen deseado.
/// El orden de los slots es irrelevante (confirmado en spec, sección 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub counts: HashMap<Gene, u8>,
}

impl Target {
    /// Crea un objetivo a partir de pares (gen, cantidad). Los genes no listados
    /// se asumen en cantidad 0 (irrelevantes/libres) salvo que se quiera exigir
    /// explícitamente 0 de un tipo — en ese caso, inclúyelo con valor 0.
    pub fn from_counts(pairs: &[(Gene, u8)]) -> Self {
        let mut counts = HashMap::new();
        for (g, n) in pairs {
            counts.insert(*g, *n);
        }
        Target { counts }
    }

    /// true si el conteo dado satisface exactamente el objetivo en todos los tipos mencionados.
    fn is_exact_match(&self, actual: &HashMap<Gene, u8>) -> bool {
        Gene::ALL.iter().all(|gene| {
            let want = *self.counts.get(gene).unwrap_or(&0);
            let have = *actual.get(gene).unwrap_or(&0);
            // Si el objetivo no menciona el gen, no se exige ese conteo (comodín).
            !self.counts.contains_key(gene) || want == have
        })
    }
}

/// Una combinación candidata: qué planta usar como centro y cuáles como donantes,
/// junto con el resultado de cruce y qué tan bien cumple el objetivo.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub center: Plant,
    pub donors: Vec<Plant>,
    pub result: CrossbreedResult,
    /// Probabilidad de obtener una planta que cumple el objetivo exactamente
    /// (sumando todas las combinaciones de slots posibles que matchean).
    pub success_probability: f64,
    /// Número esperado de genes rojos en el resultado más probable (penalización).
    pub expected_red_genes: f64,
}

/// Parámetros de configuración de la búsqueda, expuestos en la UI como "Options".
#[derive(Debug, Clone)]
pub struct SolverOptions {
    /// Máximo de plantas donantes a considerar por combinación (límite físico: 8).
    pub max_donors: usize,
    /// Mínimo de donantes a considerar (normalmente 1).
    pub min_donors: usize,
    /// Cuántos candidatos top devolver.
    pub top_n: usize,
}

impl Default for SolverOptions {
    fn default() -> Self {
        // max_donors=4 por defecto: balance entre cobertura y velocidad interactiva.
        // El límite físico real del juego es 8, pero subir el rango de búsqueda es
        // computacionalmente costoso (confirmado en la guía de rustbreeder.com: "the
        // tradeoff here is that doing the additional comparisons is very costly").
        // Se expone como opción en la UI para que el usuario lo suba si lo necesita.
        SolverOptions {
            max_donors: 4,
            min_donors: 1,
            top_n: 10,
        }
    }
}

/// Genera todas las combinaciones (sin repetición) de tamaño `k` de un slice.
fn combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 || k > items.len() {
        return vec![];
    }
    let mut result = Vec::new();
    let mut indices: Vec<usize> = (0..k).collect();
    loop {
        result.push(indices.iter().map(|&i| items[i].clone()).collect());

        // Avanza al siguiente conjunto de índices (algoritmo estándar de combinaciones).
        let mut i = k;
        loop {
            if i == 0 {
                return result;
            }
            i -= 1;
            if indices[i] != i + items.len() - k {
                break;
            }
        }
        indices[i] += 1;
        for j in (i + 1)..k {
            indices[j] = indices[j - 1] + 1;
        }
    }
}

/// Calcula la probabilidad total de que el cruce produzca CUALQUIER combinación
/// de slots cuyo conteo de genes cumpla el objetivo exactamente.
/// (A diferencia de `probability_of_exact`, que fija el orden de slots,
/// aquí sumamos sobre todas las asignaciones de slot válidas.)
fn success_probability(result: &CrossbreedResult, target: &Target) -> f64 {
    // Para cada slot, lista de (gen, prob) posibles.
    let slot_options: Vec<Vec<(Gene, f64)>> = result
        .outcomes
        .iter()
        .map(|o| o.probabilities.iter().map(|(g, p)| (*g, *p)).collect())
        .collect();

    let mut total = 0.0;
    // Backtracking sobre las (pocas) combinaciones posibles de slots con azar.
    fn recurse(
        slot_options: &[Vec<(Gene, f64)>],
        idx: usize,
        acc_prob: f64,
        acc_counts: &mut HashMap<Gene, u8>,
        target: &Target,
        total: &mut f64,
    ) {
        if idx == slot_options.len() {
            if target.is_exact_match(acc_counts) {
                *total += acc_prob;
            }
            return;
        }
        for (gene, p) in &slot_options[idx] {
            *acc_counts.entry(*gene).or_insert(0) += 1;
            recurse(slot_options, idx + 1, acc_prob * p, acc_counts, target, total);
            *acc_counts.get_mut(gene).unwrap() -= 1;
        }
    }

    let mut counts = HashMap::new();
    recurse(&slot_options, 0, 1.0, &mut counts, target, &mut total);
    total
}

fn expected_red_genes(result: &CrossbreedResult) -> f64 {
    result
        .outcomes
        .iter()
        .map(|o| {
            o.probabilities
                .iter()
                .filter(|(g, _)| g.is_red())
                .map(|(_, p)| p)
                .sum::<f64>()
        })
        .sum()
}

/// Evalúa un único centro contra el resto del pool, devolviendo todas las
/// combinaciones de donantes con probabilidad de éxito > 0.
fn evaluate_center(
    center: Plant,
    rest: &[Plant],
    target: &Target,
    options: &SolverOptions,
) -> Vec<Candidate> {
    let mut local_candidates = Vec::new();
    let max_k = options.max_donors.min(rest.len());
    let mut found_perfect_at: Option<usize> = None;

    for k in options.min_donors..=max_k {
        // Poda: si ya hay un match perfecto con menos donantes, no seguir creciendo.
        if let Some(perfect_k) = found_perfect_at {
            if k > perfect_k {
                break;
            }
        }

        for donor_set in combinations(rest, k) {
            let result = crossbreed(&center, &donor_set);
            let prob = success_probability(&result, target);
            if prob > 0.0 {
                if (prob - 1.0).abs() < 1e-9 && found_perfect_at.is_none() {
                    found_perfect_at = Some(k);
                }
                let expected_red = expected_red_genes(&result);
                local_candidates.push(Candidate {
                    center,
                    donors: donor_set,
                    result,
                    success_probability: prob,
                    expected_red_genes: expected_red,
                });
            }
        }
    }
    local_candidates
}

/// Construye, para cada índice de centro, el subconjunto del pool sin ese centro.
fn rest_without(pool: &[Plant], center_idx: usize) -> Vec<Plant> {
    pool.iter()
        .enumerate()
        .filter(|(i, _)| *i != center_idx)
        .map(|(_, p)| *p)
        .collect()
}

/// Busca las mejores combinaciones (centro + donantes) dentro de `pool` para
/// alcanzar `target`, dentro de los límites de `options`.
///
/// Optimizaciones:
/// 1. Paralelización por planta-centro (vía rayon, solo con feature `parallel`)
///    — cada centro es independiente. En WASM single-thread cae a secuencial.
/// 2. Poda incremental: una vez que un centro encuentra una combinación con
///    probabilidad 1.0 en tamaño k, no se prueban tamaños mayores para ese centro.
pub fn find_best_crosses(pool: &[Plant], target: &Target, options: &SolverOptions) -> Vec<Candidate> {
    #[cfg(feature = "parallel")]
    let per_center: Vec<Vec<Candidate>> = pool
        .par_iter()
        .enumerate()
        .map(|(center_idx, &center)| {
            let rest = rest_without(pool, center_idx);
            evaluate_center(center, &rest, target, options)
        })
        .collect();

    #[cfg(not(feature = "parallel"))]
    let per_center: Vec<Vec<Candidate>> = pool
        .iter()
        .enumerate()
        .map(|(center_idx, &center)| {
            let rest = rest_without(pool, center_idx);
            evaluate_center(center, &rest, target, options)
        })
        .collect();

    let mut candidates: Vec<Candidate> = per_center.into_iter().flatten().collect();

    // Orden: mayor probabilidad de éxito primero; en empate, menor cantidad
    // esperada de genes rojos; en empate, menor cantidad de donantes (más simple).
    candidates.sort_by(|a, b| {
        b.success_probability
            .partial_cmp(&a.success_probability)
            .unwrap()
            .then(
                a.expected_red_genes
                    .partial_cmp(&b.expected_red_genes)
                    .unwrap(),
            )
            .then(a.donors.len().cmp(&b.donors.len()))
    });

    candidates.truncate(options.top_n);
    candidates
}

// ===================== Rutas GEN.2 (dos rondas de cruce) =====================

/// Una receta de dos pasos: primero se fabrica un clon intermedio ("bridge")
/// de forma determinista, y luego ese bridge se usa como donante para alcanzar
/// el objetivo en una segunda ronda.
#[derive(Debug, Clone)]
pub struct TwoStepRecipe {
    /// Paso 1: el cruce que fabrica el bridge (siempre determinista).
    pub step1: Candidate,
    /// El clon intermedio que produce el paso 1.
    pub bridge: Plant,
    /// Paso 2: el cruce final que usa el bridge para alcanzar el objetivo.
    pub step2: Candidate,
    /// Probabilidad total = prob(paso1) * prob(paso2). Como el bridge es
    /// determinista (prob 1.0), equivale a la probabilidad del paso 2.
    pub total_probability: f64,
}

/// Genera el conjunto de "bridges" deterministas y útiles alcanzables en una
/// ronda desde el pool. Un bridge es útil si es genéticamente distinto de todas
/// las plantas que ya tenemos (no tiene sentido fabricar algo que ya posees).
fn reachable_bridges(pool: &[Plant], options: &SolverOptions, limit: usize) -> Vec<(Plant, Candidate)> {
    let mut seen: std::collections::HashSet<[Gene; crate::plant::SLOT_COUNT]> =
        pool.iter().map(|p| p.slots).collect();
    let mut bridges: Vec<(Plant, Candidate)> = Vec::new();

    for (center_idx, &center) in pool.iter().enumerate() {
        let rest = rest_without(pool, center_idx);
        let max_k = options.max_donors.min(rest.len());

        for k in options.min_donors..=max_k {
            for donor_set in combinations(&rest, k) {
                let result = crossbreed(&center, &donor_set);
                // Solo bridges 100% deterministas (garantizan la cadena GEN.2).
                if let Some(bridge_plant) = result.as_plant() {
                    if seen.insert(bridge_plant.slots) {
                        let expected_red = expected_red_genes(&result);
                        bridges.push((
                            bridge_plant,
                            Candidate {
                                center,
                                donors: donor_set,
                                result,
                                success_probability: 1.0,
                                expected_red_genes: expected_red,
                            },
                        ));
                    }
                }
            }
        }
    }

    // Prioriza bridges con menos genes rojos (más útiles para god clones) y
    // limita la cantidad para acotar el coste de la segunda ronda.
    bridges.sort_by(|a, b| {
        a.0.red_count()
            .cmp(&b.0.red_count())
            .then(a.1.donors.len().cmp(&b.1.donors.len()))
    });
    bridges.truncate(limit);
    bridges
}

/// Busca rutas GEN.2: cuando el objetivo no se alcanza al 100% en una sola ronda,
/// intenta fabricar un clon intermedio que sí permita alcanzarlo en la segunda.
///
/// Devuelve las mejores recetas de dos pasos, ordenadas por probabilidad total.
/// Solo considera bridges deterministas, de modo que la fiabilidad de la receta
/// depende únicamente de la segunda ronda (coherente con la recomendación de la
/// comunidad de usar GEN.2 para obtener rutas estables).
pub fn find_two_step_recipes(
    pool: &[Plant],
    target: &Target,
    options: &SolverOptions,
) -> Vec<TwoStepRecipe> {
    // Límite de bridges a explorar — acota la explosión combinatoria.
    const BRIDGE_LIMIT: usize = 40;

    let bridges = reachable_bridges(pool, options, BRIDGE_LIMIT);
    let mut recipes: Vec<TwoStepRecipe> = Vec::new();

    for (bridge, step1) in bridges {
        // Pool ampliado con el bridge fabricado.
        let mut extended = pool.to_vec();
        extended.push(bridge);

        // Segunda ronda: buscar el mejor cruce que use el bridge para llegar al target.
        let second = find_best_crosses(&extended, target, options);
        for cand in second {
            // La receta solo es GEN.2 genuina si el bridge participa realmente
            // en el cruce final (como centro o como donante).
            let bridge_used =
                cand.center == bridge || cand.donors.iter().any(|d| *d == bridge);
            if bridge_used && cand.success_probability > 0.0 {
                let total = step1.success_probability * cand.success_probability;
                recipes.push(TwoStepRecipe {
                    step1: step1.clone(),
                    bridge,
                    step2: cand,
                    total_probability: total,
                });
                break; // mejor receta por bridge basta
            }
        }
    }

    recipes.sort_by(|a, b| {
        b.total_probability
            .partial_cmp(&a.total_probability)
            .unwrap()
            .then(
                (a.step1.donors.len() + a.step2.donors.len())
                    .cmp(&(b.step1.donors.len() + b.step2.donors.len())),
            )
    });
    recipes.truncate(options.top_n);
    recipes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gene::Gene::*;

    #[test]
    fn combinations_generates_correct_count() {
        let items = vec![1, 2, 3, 4];
        let combos = combinations(&items, 2);
        // C(4,2) = 6
        assert_eq!(combos.len(), 6);
    }

    #[test]
    fn combinations_of_size_zero_or_too_large_are_empty() {
        let items = vec![1, 2, 3];
        assert_eq!(combinations(&items, 0).len(), 0);
        assert_eq!(combinations(&items, 5).len(), 0);
    }

    #[test]
    fn finds_perfect_deterministic_match() {
        // Centro X-todo, donantes que fuerzan GGGYYY de forma determinista en cada slot.
        let center = Plant::parse("XXXXXX").unwrap();
        let d1 = Plant::parse("GGGYYY").unwrap();
        let d2 = Plant::parse("GGGYYY").unwrap();
        let pool = vec![center, d1, d2];

        let target = Target::from_counts(&[(G, 3), (Y, 3)]);
        let options = SolverOptions::default();

        let results = find_best_crosses(&pool, &target, &options);
        assert!(!results.is_empty());
        let best = &results[0];
        assert!((best.success_probability - 1.0).abs() < 1e-9);
    }

    #[test]
    fn no_match_possible_returns_empty() {
        // Pool de solo genes rojos no puede nunca producir un objetivo todo verde.
        let center = Plant::parse("XXXXXX").unwrap();
        let d1 = Plant::parse("WWWWWW").unwrap();
        let pool = vec![center, d1];

        let target = Target::from_counts(&[(G, 6)]);
        let options = SolverOptions::default();

        let results = find_best_crosses(&pool, &target, &options);
        assert!(results.is_empty());
    }

    #[test]
    fn respects_max_donors_limit() {
        let center = Plant::parse("XXXXXX").unwrap();
        let donors: Vec<Plant> = (0..10)
            .map(|_| Plant::parse("GGGYYY").unwrap())
            .collect();
        let mut pool = vec![center];
        pool.extend(donors);

        let target = Target::from_counts(&[(G, 3), (Y, 3)]);
        let options = SolverOptions {
            max_donors: 2,
            min_donors: 1,
            top_n: 100,
        };

        let results = find_best_crosses(&pool, &target, &options);
        for c in &results {
            assert!(c.donors.len() <= 2);
        }
    }

    #[test]
    fn two_step_finds_recipe_using_fabricated_bridge() {
        // Pool verificado donde el motor GEN.2 produce al menos una receta válida.
        // El donante limpio GGGYYY (verde en slots 0-5) puede cambiar el slot 6 de
        // un centro sin destruir sus otros genes, permitiendo rutas de dos pasos.
        let pool = vec![
            Plant::parse("GGGYYX").unwrap(),
            Plant::parse("GGGYYY").unwrap(),
            Plant::parse("XXXXXY").unwrap(),
            Plant::parse("XXXXXY").unwrap(),
        ];
        let target = Target::from_counts(&[(G, 3), (Y, 3)]);
        let options = SolverOptions::default();

        let recipes = find_two_step_recipes(&pool, &target, &options);
        assert!(
            !recipes.is_empty(),
            "este pool debería producir al menos una receta GEN.2"
        );
        // Toda receta debe cumplir las invariantes del algoritmo.
        for r in &recipes {
            assert!((r.step1.success_probability - 1.0).abs() < 1e-9);
            let used = r.step2.center == r.bridge
                || r.step2.donors.iter().any(|d| *d == r.bridge);
            assert!(used, "el bridge debe participar en el paso 2");
            assert!(
                (r.total_probability - r.step2.success_probability).abs() < 1e-9
            );
            // El bridge nunca es una planta que ya estaba en el pool original.
            assert!(!pool.contains(&r.bridge));
        }
    }

    #[test]
    fn two_step_bridge_is_genetically_new() {
        // El bridge fabricado no debe ser idéntico a ninguna planta del pool
        // original (no tiene sentido "fabricar" algo que ya tienes).
        let pool = vec![
            Plant::parse("XXXXXX").unwrap(),
            Plant::parse("GGGXXX").unwrap(),
            Plant::parse("YYYXXX").unwrap(),
            Plant::parse("GGGXXX").unwrap(),
            Plant::parse("YYYXXX").unwrap(),
        ];
        let target = Target::from_counts(&[(G, 3), (Y, 3)]);
        let options = SolverOptions::default();

        let recipes = find_two_step_recipes(&pool, &target, &options);
        for r in &recipes {
            assert!(
                !pool.contains(&r.bridge),
                "el bridge {:?} no debería estar ya en el pool",
                r.bridge.to_string()
            );
        }
    }
}

#[cfg(test)]
mod perf_tests {
    use super::*;
    use crate::gene::Gene::*;
    use std::time::Instant;

    #[test]
    fn realistic_pool_size_completes_quickly() {
        // Simula un pool realista: 18 clones variados (rango típico citado en guías: 10-20).
        let raw = [
            "GGGYYH", "YYXXGY", "GHGYYW", "WHGGYH", "WGHYGH", "YYYWGH",
            "WGHGYG", "YHGWYG", "GYGYWH", "GGYYHH", "YGYGYG", "HGYGYH",
            "GYHWXG", "YHYXGY", "GHGYYW", "WHGGYH", "GGGGGG", "YYYYYY",
        ];
        let pool: Vec<Plant> = raw.iter().map(|s| Plant::parse(s).unwrap()).collect();
        assert_eq!(pool.len(), 18);

        let target = Target::from_counts(&[(G, 3), (Y, 3)]);
        let options = SolverOptions {
            max_donors: 8,
            min_donors: 1,
            top_n: 10,
        };

        let start = Instant::now();
        let results = find_best_crosses(&pool, &target, &options);
        let elapsed = start.elapsed();

        println!("Pool de 18 plantas, hasta 8 donantes: {:?} en {:?}", results.len(), elapsed);
        // No debe tardar más de unos pocos segundos en hardware modesto.
        assert!(elapsed.as_secs() < 30, "tardó demasiado: {:?}", elapsed);
    }
}
