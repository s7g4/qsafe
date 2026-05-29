//! User interface module for Q-Safe

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "web")]
use web_sys::{console, window};

#[cfg(feature = "visualization")]
use plotters::prelude::*;

pub struct UI {
    #[cfg(feature = "web")]
    console: web_sys::console::Console,
}

impl UI {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "web")]
            console: web_sys::console::Console::new().unwrap(),
        }
    }

    /// Display a message in the console
    pub fn log(&self, message: &str) {
        #[cfg(feature = "web")]
        {
            self.console.log_1(&JsValue::from_str(message));
        }

        #[cfg(not(feature = "web"))]
        {
            println!("{}", message);
        }
    }

    /// Display quantum key exchange visualization
    #[cfg(feature = "visualization")]
    pub fn visualize_qkd(&self, key_exchange_data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("qkd_visualization.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Quantum Key Distribution", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0f64..key_exchange_data.len() as f64, 0f64..1f64)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            key_exchange_data.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &RED,
        ))?;

        Ok(())
    }

    /// Simulate quantum handshake animation
    pub fn animate_quantum_handshake(&self) {
        self.log("Initiating Quantum Handshake...");
        self.log("Preparing quantum bits...");
        self.log("Measuring in random bases...");
        self.log("Detecting eavesdropping...");
        self.log("Key exchange successful!");
        self.log("Secure channel established.");
    }

    /// Display chat interface (simplified text-based)
    pub fn display_chat(&self, messages: &[String]) {
        self.log("=== Q-Safe Secure Chat ===");
        for message in messages {
            self.log(&format!("> {}", message));
        }
        self.log("=========================");
    }

    /// Get user input (simplified)
    pub fn get_user_input(&self, prompt: &str) -> String {
        self.log(&format!("{}: ", prompt));
        // In a real implementation, this would read from stdin or web input
        String::new()
    }
}

#[cfg(feature = "web")]
#[wasm_bindgen]
impl UI {
    #[wasm_bindgen(constructor)]
    pub fn new_web() -> UI {
        Self::new()
    }

    #[wasm_bindgen]
    pub fn log_web(&self, message: &str) {
        self.log(message);
    }

    #[wasm_bindgen]
    pub fn animate_handshake_web(&self) {
        self.animate_quantum_handshake();
    }
}
