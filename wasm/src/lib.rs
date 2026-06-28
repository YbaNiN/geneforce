//! Bindings WASM para el núcleo de crossbreeding.
//!
//! Expone a JavaScript:
//! - `validate_plant(s)`: valida una cadena de genes, devuelve error legible o null.
//! - `crossbreed(center, donors)`: cruza y devuelve el resultado por slot.
//! - `solve(request)`: ejecuta el solver completo y devuelve los mejores candidatos.
//!
//! Los datos se intercambian como objetos JS (vía serde-wasm-bindgen) para una
//! API ergonómica desde el frontend, sin que JS tenga que conocer el layout interno.

use geneforge_core::crossbreed::crossbreed as core_crossbreed;
use geneforge_core::gene::Gene;
use geneforge_core::plant::Plant;
use geneforge_core::solver::{find_best_crosses, SolverOptions, Target};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Inicialización: instala un hook para que los panics de Rust aparezcan en la
/// consola del navegador en vez de un críptico "unreachable executed".
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ----- Tipos de intercambio (serializables a/desde JS) -----

#[derive(Serialize, Deserialize)]
pub struct GeneProb {
    pub gene: String,
    pub probability: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SlotResult {
    /// Posibles genes resultantes en este slot, con su probabilidad.
    pub options: Vec<GeneProb>,
    /// true si el slot es 100% determinista.
    pub certain: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CrossResult {
    pub slots: Vec<SlotResult>,
    pub fully_deterministic: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TargetSpec {
    /// Pares (gen, cantidad), ej. [("G",3),("Y",3)]. Genes no listados = comodín.
    pub counts: Vec<(String, u8)>,
}

#[derive(Serialize, Deserialize)]
pub struct SolveRequest {
    /// Plantas disponibles, como cadenas de 6 genes (ej. "GGGYYH").
    pub pool: Vec<String>,
    pub target: TargetSpec,
    #[serde(default = "default_max_donors")]
    pub max_donors: usize,
    #[serde(default = "default_min_donors")]
    pub min_donors: usize,
    #[serde(default = "default_top_n")]
    pub top_n: usize,
    /// Si true, cuando GEN.1 no logre un match perfecto se buscan rutas GEN.2.
    #[serde(default)]
    pub allow_two_gen: bool,
}

fn default_max_donors() -> usize {
    4
}
fn default_min_donors() -> usize {
    1
}
fn default_top_n() -> usize {
    10
}

#[derive(Serialize, Deserialize)]
pub struct CandidateOut {
    pub center: String,
    pub donors: Vec<String>,
    pub result: CrossResult,
    pub success_probability: f64,
    pub expected_red_genes: f64,
    /// Número de donantes (conveniencia para la UI).
    pub donor_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TwoStepOut {
    /// Paso 1: el cruce que fabrica el clon intermedio (siempre determinista).
    pub step1: CandidateOut,
    /// El clon intermedio fabricado.
    pub bridge: String,
    /// Paso 2: el cruce final que usa el bridge para alcanzar el objetivo.
    pub step2: CandidateOut,
    pub total_probability: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SolveResponse {
    pub candidates: Vec<CandidateOut>,
    /// Recetas de dos pasos (GEN.2), presentes solo si se pidió `allow_two_gen`
    /// y GEN.1 no alcanzó un match perfecto.
    pub two_step: Vec<TwoStepOut>,
    /// Plantas del pool que no se pudieron parsear (cadena -> mensaje de error).
    pub invalid: Vec<InvalidPlant>,
}

#[derive(Serialize, Deserialize)]
pub struct InvalidPlant {
    pub input: String,
    pub error: String,
}

// ----- Conversiones internas -----

fn cross_result_to_out(result: &geneforge_core::crossbreed::CrossbreedResult) -> CrossResult {
    let slots = result
        .outcomes
        .iter()
        .map(|o| {
            let options = o
                .probabilities
                .iter()
                .map(|(g, p)| GeneProb {
                    gene: g.to_string(),
                    probability: *p,
                })
                .collect();
            SlotResult {
                options,
                certain: o.is_certain(),
            }
        })
        .collect();
    CrossResult {
        slots,
        fully_deterministic: result.is_fully_deterministic(),
    }
}

fn parse_target(spec: &TargetSpec) -> Result<Target, String> {
    let mut pairs = Vec::new();
    for (gene_str, count) in &spec.counts {
        let c = gene_str
            .chars()
            .next()
            .ok_or_else(|| "gen vacío en el objetivo".to_string())?;
        let gene = Gene::from_char(c).ok_or_else(|| format!("gen inválido: '{}'", gene_str))?;
        pairs.push((gene, *count));
    }
    Ok(Target::from_counts(&pairs))
}

// ----- API pública WASM -----

/// Valida una cadena de genes. Devuelve `null` si es válida, o un mensaje de error.
#[wasm_bindgen]
pub fn validate_plant(s: &str) -> Option<String> {
    match Plant::parse(s) {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    }
}

/// Cruza un centro con donantes. Devuelve el resultado por slot.
/// `center` y `donors` son cadenas de 6 genes. Lanza error si alguna es inválida.
#[wasm_bindgen]
pub fn crossbreed(center: &str, donors: Vec<String>) -> Result<JsValue, JsValue> {
    let center_plant = Plant::parse(center).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let donor_plants: Result<Vec<Plant>, _> = donors.iter().map(|s| Plant::parse(s)).collect();
    let donor_plants = donor_plants.map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = core_crossbreed(&center_plant, &donor_plants);
    let out = cross_result_to_out(&result);
    serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn candidate_to_out(c: &geneforge_core::solver::Candidate) -> CandidateOut {
    CandidateOut {
        center: c.center.to_string(),
        donors: c.donors.iter().map(|d| d.to_string()).collect(),
        result: cross_result_to_out(&c.result),
        success_probability: c.success_probability,
        expected_red_genes: c.expected_red_genes,
        donor_count: c.donors.len(),
    }
}

/// Ejecuta el solver completo. Recibe un `SolveRequest` (objeto JS) y devuelve
/// un `SolveResponse` con los mejores candidatos y las plantas inválidas.
#[wasm_bindgen]
pub fn solve(request: JsValue) -> Result<JsValue, JsValue> {
    let req: SolveRequest =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Parseo del pool, separando válidas de inválidas (para feedback en la UI).
    let mut pool = Vec::new();
    let mut invalid = Vec::new();
    for s in &req.pool {
        match Plant::parse(s) {
            Ok(p) => pool.push(p),
            Err(e) => invalid.push(InvalidPlant {
                input: s.clone(),
                error: e.to_string(),
            }),
        }
    }

    let target = parse_target(&req.target).map_err(|e| JsValue::from_str(&e))?;

    let options = SolverOptions {
        max_donors: req.max_donors.clamp(1, 8),
        min_donors: req.min_donors.max(1),
        top_n: req.top_n.clamp(1, 100),
    };

    let gen1 = find_best_crosses(&pool, &target, &options);
    let candidates: Vec<CandidateOut> = gen1.iter().map(candidate_to_out).collect();

    // GEN.2 solo se calcula si se solicita y GEN.1 no produjo un match perfecto
    // (si ya hay una receta de un paso al 100%, no hace falta complicarse con dos).
    let has_perfect_gen1 = gen1
        .first()
        .map(|c| (c.success_probability - 1.0).abs() < 1e-9)
        .unwrap_or(false);

    let two_step: Vec<TwoStepOut> = if req.allow_two_gen && !has_perfect_gen1 {
        geneforge_core::solver::find_two_step_recipes(&pool, &target, &options)
            .iter()
            .map(|r| TwoStepOut {
                step1: candidate_to_out(&r.step1),
                bridge: r.bridge.to_string(),
                step2: candidate_to_out(&r.step2),
                total_probability: r.total_probability,
            })
            .collect()
    } else {
        Vec::new()
    };

    let response = SolveResponse {
        candidates,
        two_step,
        invalid,
    };

    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}
