#![no_main]

use libfuzzer_sys::fuzz_target;
use simulation_framework::config::SimulationConfig;

fuzz_target!(|data: &[u8]| {
    // Try to parse the fuzzer input as YAML config
    // This tests the robustness of YAML config parsing
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_yaml::from_str::<SimulationConfig>(s);
    }
});
