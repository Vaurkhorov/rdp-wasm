use js_sys::{Float64Array, JsString};
use std::slice::Iter;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Curve {
    timestamps: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Curve {
    pub fn debug_print(&self) {
        log(format!("{:#?}", self).as_str());
    }

    pub fn get_timestamps(&self) -> Float64Array {
        Float64Array::from(&self.timestamps[..])
    }

    pub fn get_x(&self) -> Float64Array {
        Float64Array::from(&self.x[..])
    }

    pub fn get_y(&self) -> Float64Array {
        Float64Array::from(&self.y[..])
    }

    pub fn get_csv(&self) -> JsString {
        let mut output = String::from("timestamps, x, y\n");

        for i in 0..self.x.len() {
            output.push_str(&format!(
                "{}, {}, {}\n",
                self.timestamps[i], self.x[i], self.y[i]
            ))
        }

        JsString::from(output)
    }
}

impl Curve {
    pub fn new() -> Self {
        Curve {
            timestamps: Vec::new(),
            x: Vec::new(),
            y: Vec::new(),
        }
    }

    pub fn from_vectors(timestamps: Vec<f64>, x: Vec<f64>, y: Vec<f64>) -> Self {
        Curve { timestamps, x, y }
    }

    pub fn get_iters(&self) -> (Iter<f64>, Iter<f64>, Iter<f64>) {
        (self.timestamps.iter(), self.x.iter(), self.y.iter())
    }

    pub fn insert(&mut self, index: usize, timestamp: f64, x: f64, y: f64) {
        self.timestamps.insert(index, timestamp);
        self.x.insert(index, x);
        self.y.insert(index, y);
    }

    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    pub fn assert_curve(actual: &Curve, expected: &Curve) {
        assert_eq!(actual.timestamps, expected.timestamps);
        assert_eq!(actual.x, expected.x);
        assert_eq!(actual.y, expected.y);
    }
}
