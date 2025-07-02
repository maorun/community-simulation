use clap::Parser;
use simulation_framework::{SimulationConfig, SimulationEngine, SimulationResult};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "simulation")]
#[command(about = "High-performance simulation framework")]
struct Args {
    #[arg(short, long, default_value = "1000")]
        steps: usize,
            
            #[arg(short, long, default_value = "10000")]
                    entities: usize,
                        
                            #[arg(short, long)]
                                output: Option<String>,
                                    
                                        #[arg(long, default_value = "4")]
                                            threads: usize,
                                            }

                                            fn main() -> Result<(), Box<dyn std::error::Error>> {
                                                let args = Args::parse();
                                                    
                                                        rayon::ThreadPoolBuilder::new()
                                                                .num_threads(args.threads)
                                                                        .build_global()?;
                                                                            
                                                                                let config = SimulationConfig {
                                                                                        max_steps: args.steps,
                                                                                                entity_count: args.entities,
                                                                                                        time_step: 0.01,
                                                                                                                seed: 42,
                                                                                                                    };
                                                                                                                        
                                                                                                                            println!("Initializing simulation with {} entities for {} steps", 
                                                                                                                                         config.entity_count, config.max_steps);
                                                                                                                                             
                                                                                                                                                 let start_time = Instant::now();
                                                                                                                                                     let mut engine = SimulationEngine::new(config);
                                                                                                                                                         let result = engine.run();
                                                                                                                                                             let duration = start_time.elapsed();
                                                                                                                                                                 
                                                                                                                                                                     println!("Simulation completed in {:.2}s", duration.as_secs_f64());
                                                                                                                                                                         println!("Performance: {:.0} steps/second", 
                                                                                                                                                                                      args.steps as f64 / duration.as_secs_f64());
                                                                                                                                                                                          
                                                                                                                                                                                              if let Some(output_path) = args.output {
                                                                                                                                                                                                      result.save_to_file(&output_path)?;
                                                                                                                                                                                                              println!("Results saved to {}", output_path);
                                                                                                                                                                                                                  }
                                                                                                                                                                                                                      
                                                                                                                                                                                                                          result.print_summary();
                                                                                                                                                                                                                              
                                                                                                                                                                                                                                  Ok(())
                                                                                                                                                                                                                                  }