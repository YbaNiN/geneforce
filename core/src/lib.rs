//! Núcleo de dominio para el simulador de crossbreeding de Rust (el juego).
//!
//! Módulos:
//! - `gene`: tipos de gen y sus pesos.
//! - `plant`: representación de una planta (6 slots).
//! - `crossbreed`: lógica de cruce centro-vs-donantes.
//! - `solver`: búsqueda de la mejor combinación de plantas para un objetivo.

pub mod crossbreed;
pub mod gene;
pub mod plant;
pub mod solver;

pub use crossbreed::{crossbreed as cross, CrossbreedResult, SlotOutcome};
pub use gene::{Gene, GeneColor};
pub use plant::{ParsePlantError, Plant, SLOT_COUNT};
