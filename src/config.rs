use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub max_steps: usize,
        pub entity_count: usize,
            pub time_step: f64,
                pub seed: u64,
                }

                impl Default for SimulationConfig {
                    fn default() -> Self {
                            Self {
                                        max_steps: 1000,
                                                    entity_count: 1000,
                                                                time_step: 0.01,
                                                                            seed: 42,
                                                                                    }
                                                                                        }
                                                                                        }
                