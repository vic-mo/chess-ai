//! Runtime-tunable search parameters for SPSA optimization.
//!
//! This module provides thread-local storage for search parameters that can be
//! modified via UCI setoption commands for automated tuning with SPSA.

use std::cell::RefCell;
use std::sync::OnceLock;

thread_local! {
    /// Thread-local storage for tunable search parameters.
    /// When set, the search functions will use these parameters instead of defaults.
    static SEARCH_PARAMS: RefCell<SearchParams> = RefCell::new(SearchParams::default());
}

/// Set the thread-local search parameters.
pub fn set_search_params(params: SearchParams) {
    SEARCH_PARAMS.with(|p| *p.borrow_mut() = params);
}

/// Get a copy of the current search parameters.
pub fn get_search_params() -> SearchParams {
    SEARCH_PARAMS.with(|p| p.borrow().clone())
}

/// Update a single parameter by name.
pub fn set_param(name: &str, value: i32) -> Result<(), String> {
    SEARCH_PARAMS.with(|p| {
        let mut params = p.borrow_mut();
        params.set_by_name(name, value)
    })
}

/// Get a specific parameter value by name.
pub fn get_param(name: &str) -> Result<i32, String> {
    SEARCH_PARAMS.with(|p| {
        let params = p.borrow();
        params.get_by_name(name)
    })
}

/// Tunable search parameters.
///
/// These parameters control search behavior and can be optimized via SPSA.
#[derive(Debug, Clone)]
pub struct SearchParams {
    // Late Move Reduction (LMR)
    pub lmr_base_reduction: i32,           // Base reduction for LMR (1-3)
    pub lmr_move_threshold: usize,         // Moves before LMR kicks in (3-6)
    pub lmr_depth_threshold: i32,          // Depth before LMR kicks in (2-4)

    // Null Move Pruning
    pub null_move_r: i32,                  // Null move reduction factor (2-3)
    pub null_move_min_depth: i32,          // Minimum depth for null move (2-4)

    // Futility Pruning margins by depth
    pub futility_margin_d1: i32,           // Depth 1 margin (50-150)
    pub futility_margin_d2: i32,           // Depth 2 margin (150-250)
    pub futility_margin_d3: i32,           // Depth 3 margin (250-350)

    // Reverse Futility Pruning margins by depth
    pub rfp_margin_d1: i32,                // Depth 1 margin (50-150)
    pub rfp_margin_d2: i32,                // Depth 2 margin (150-250)
    pub rfp_margin_d3: i32,                // Depth 3 margin (250-350)
    pub rfp_margin_d4: i32,                // Depth 4 margin (350-450)
    pub rfp_margin_d5: i32,                // Depth 5 margin (450-550)

    // Razoring margins by depth
    pub razor_margin_d1: i32,              // Depth 1 margin (150-250)
    pub razor_margin_d2: i32,              // Depth 2 margin (250-350)
    pub razor_margin_d3: i32,              // Depth 3 margin (350-450)

    // Late Move Pruning thresholds by depth
    pub lmp_threshold_d1: usize,           // Depth 1 threshold (2-5)
    pub lmp_threshold_d2: usize,           // Depth 2 threshold (4-8)
    pub lmp_threshold_d3: usize,           // Depth 3 threshold (8-16)

    // Aspiration Windows
    pub aspiration_delta: i32,             // Initial window size (30-80)

    // Internal Iterative Deepening/Reduction
    pub iid_depth_reduction: i32,          // IID depth reduction (1-3)
    pub iir_depth_reduction: i32,          // IIR depth reduction (1-2)
    pub iid_min_depth: i32,                // Minimum depth for IID (3-5)

    // Singular Extensions
    pub singular_margin: i32,              // Margin for singularity (50-150)
    pub singular_depth_reduction: i32,     // Depth reduction for verification (2-4)
    pub singular_min_depth: i32,           // Minimum depth for singular (6-10)

    // Evaluation scaling
    pub king_safety_divisor: i32,          // King safety scaling (8-16)
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            // LMR - current simple formula: if move >= 6 && depth >= 6 then 2 else 1
            lmr_base_reduction: 2,
            lmr_move_threshold: 6,
            lmr_depth_threshold: 6,

            // Null move
            null_move_r: 2,
            null_move_min_depth: 3,

            // Futility pruning
            futility_margin_d1: 100,
            futility_margin_d2: 200,
            futility_margin_d3: 300,

            // Reverse futility pruning
            rfp_margin_d1: 100,
            rfp_margin_d2: 200,
            rfp_margin_d3: 300,
            rfp_margin_d4: 400,
            rfp_margin_d5: 500,

            // Razoring
            razor_margin_d1: 200,
            razor_margin_d2: 300,
            razor_margin_d3: 400,

            // Late move pruning
            lmp_threshold_d1: 3,
            lmp_threshold_d2: 6,
            lmp_threshold_d3: 12,

            // Aspiration
            aspiration_delta: 50,

            // IID/IIR
            iid_depth_reduction: 2,
            iir_depth_reduction: 1,
            iid_min_depth: 4,

            // Singular extensions
            singular_margin: 100,
            singular_depth_reduction: 4,
            singular_min_depth: 8,

            // Evaluation
            king_safety_divisor: 12,
        }
    }
}

impl SearchParams {
    /// Set a parameter by name.
    pub fn set_by_name(&mut self, name: &str, value: i32) -> Result<(), String> {
        match name {
            "lmr_base_reduction" => self.lmr_base_reduction = value,
            "lmr_move_threshold" => self.lmr_move_threshold = value as usize,
            "lmr_depth_threshold" => self.lmr_depth_threshold = value,

            "null_move_r" => self.null_move_r = value,
            "null_move_min_depth" => self.null_move_min_depth = value,

            "futility_margin_d1" => self.futility_margin_d1 = value,
            "futility_margin_d2" => self.futility_margin_d2 = value,
            "futility_margin_d3" => self.futility_margin_d3 = value,

            "rfp_margin_d1" => self.rfp_margin_d1 = value,
            "rfp_margin_d2" => self.rfp_margin_d2 = value,
            "rfp_margin_d3" => self.rfp_margin_d3 = value,
            "rfp_margin_d4" => self.rfp_margin_d4 = value,
            "rfp_margin_d5" => self.rfp_margin_d5 = value,

            "razor_margin_d1" => self.razor_margin_d1 = value,
            "razor_margin_d2" => self.razor_margin_d2 = value,
            "razor_margin_d3" => self.razor_margin_d3 = value,

            "lmp_threshold_d1" => self.lmp_threshold_d1 = value as usize,
            "lmp_threshold_d2" => self.lmp_threshold_d2 = value as usize,
            "lmp_threshold_d3" => self.lmp_threshold_d3 = value as usize,

            "aspiration_delta" => self.aspiration_delta = value,

            "iid_depth_reduction" => self.iid_depth_reduction = value,
            "iir_depth_reduction" => self.iir_depth_reduction = value,
            "iid_min_depth" => self.iid_min_depth = value,

            "singular_margin" => self.singular_margin = value,
            "singular_depth_reduction" => self.singular_depth_reduction = value,
            "singular_min_depth" => self.singular_min_depth = value,

            "king_safety_divisor" => self.king_safety_divisor = value,

            _ => return Err(format!("Unknown parameter: {}", name)),
        }
        Ok(())
    }

    /// Get a parameter value by name.
    pub fn get_by_name(&self, name: &str) -> Result<i32, String> {
        match name {
            "lmr_base_reduction" => Ok(self.lmr_base_reduction),
            "lmr_move_threshold" => Ok(self.lmr_move_threshold as i32),
            "lmr_depth_threshold" => Ok(self.lmr_depth_threshold),

            "null_move_r" => Ok(self.null_move_r),
            "null_move_min_depth" => Ok(self.null_move_min_depth),

            "futility_margin_d1" => Ok(self.futility_margin_d1),
            "futility_margin_d2" => Ok(self.futility_margin_d2),
            "futility_margin_d3" => Ok(self.futility_margin_d3),

            "rfp_margin_d1" => Ok(self.rfp_margin_d1),
            "rfp_margin_d2" => Ok(self.rfp_margin_d2),
            "rfp_margin_d3" => Ok(self.rfp_margin_d3),
            "rfp_margin_d4" => Ok(self.rfp_margin_d4),
            "rfp_margin_d5" => Ok(self.rfp_margin_d5),

            "razor_margin_d1" => Ok(self.razor_margin_d1),
            "razor_margin_d2" => Ok(self.razor_margin_d2),
            "razor_margin_d3" => Ok(self.razor_margin_d3),

            "lmp_threshold_d1" => Ok(self.lmp_threshold_d1 as i32),
            "lmp_threshold_d2" => Ok(self.lmp_threshold_d2 as i32),
            "lmp_threshold_d3" => Ok(self.lmp_threshold_d3 as i32),

            "aspiration_delta" => Ok(self.aspiration_delta),

            "iid_depth_reduction" => Ok(self.iid_depth_reduction),
            "iir_depth_reduction" => Ok(self.iir_depth_reduction),
            "iid_min_depth" => Ok(self.iid_min_depth),

            "singular_margin" => Ok(self.singular_margin),
            "singular_depth_reduction" => Ok(self.singular_depth_reduction),
            "singular_min_depth" => Ok(self.singular_min_depth),

            "king_safety_divisor" => Ok(self.king_safety_divisor),

            _ => Err(format!("Unknown parameter: {}", name)),
        }
    }

    /// Get all parameter names.
    pub fn param_names() -> Vec<&'static str> {
        vec![
            "lmr_base_reduction",
            "lmr_move_threshold",
            "lmr_depth_threshold",
            "null_move_r",
            "null_move_min_depth",
            "futility_margin_d1",
            "futility_margin_d2",
            "futility_margin_d3",
            "rfp_margin_d1",
            "rfp_margin_d2",
            "rfp_margin_d3",
            "rfp_margin_d4",
            "rfp_margin_d5",
            "razor_margin_d1",
            "razor_margin_d2",
            "razor_margin_d3",
            "lmp_threshold_d1",
            "lmp_threshold_d2",
            "lmp_threshold_d3",
            "aspiration_delta",
            "iid_depth_reduction",
            "iir_depth_reduction",
            "iid_min_depth",
            "singular_margin",
            "singular_depth_reduction",
            "singular_min_depth",
            "king_safety_divisor",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = SearchParams::default();
        assert_eq!(params.lmr_base_reduction, 2);
        assert_eq!(params.null_move_r, 2);
        assert_eq!(params.aspiration_delta, 50);
    }

    #[test]
    fn test_set_get_param() {
        set_param("lmr_base_reduction", 3).unwrap();
        assert_eq!(get_param("lmr_base_reduction").unwrap(), 3);

        set_param("aspiration_delta", 60).unwrap();
        assert_eq!(get_param("aspiration_delta").unwrap(), 60);
    }

    #[test]
    fn test_invalid_param() {
        assert!(set_param("invalid_param", 100).is_err());
        assert!(get_param("invalid_param").is_err());
    }

    #[test]
    fn test_all_params_accessible() {
        let names = SearchParams::param_names();
        let params = SearchParams::default();

        for name in names {
            // Should not panic
            let _ = params.get_by_name(name).unwrap();
        }
    }
}
