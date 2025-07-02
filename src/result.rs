use crate::Entity;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    pub total_steps: usize,
        pub total_duration: f64,
            pub final_entities: Vec<Entity>,
                pub step_times: Vec<f64>,
                    pub active_entities: usize,
                    }

                    impl SimulationResult {
                        pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
                                let json = serde_json::to_string_pretty(self)?;
                                        let mut file = File::create(path)?;
                                                file.write_all(json.as_bytes())?;
                                                        Ok(())
                                                            }
                                                                
                                                                    pub fn print_summary(&self) {
                                                                            println!("\n=== Simulation Summary ===");
                                                                                    println!("Total steps: {}", self.total_steps);
                                                                                            println!("Total duration: {:.2}s", self.total_duration);
                                                                                                    println!("Average step time: {:.4}ms", 
                                                                                                                     self.step_times.iter().sum::<f64>() / self.step_times.len() as f64 * 1000.0);
                                                                                                                             println!("Active entities remaining: {}", self.active_entities);
                                                                                                                                     println!("Performance: {:.0} steps/second", 
                                                                                                                                                      self.total_steps as f64 / self.total_duration);
                                                                                                                                                              
                                                                                                                                                                      if !self.step_times.is_empty() {
                                                                                                                                                                                  let min_time = self.step_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                                                                                                                                                                                              let max_time = self.step_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                                                                                                                                                                                                          println!("Step time range: {:.4}ms - {:.4}ms", min_time * 1000.0, max_time * 1000.0);
                                                                                                                                                                                                                  }
                                                                                                                                                                                                                      }
                                                                                                                                                                                                                      }