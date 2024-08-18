use wasm_bindgen::prelude::*;
use js_sys::Float64Array;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct DecimatedCurve {
    timestamps: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
}

#[wasm_bindgen]
impl DecimatedCurve {
    pub fn debug_print(&self) {
        log(format!("{:#?}", self).as_str());
    }

    #[wasm_bindgen]
    pub fn get_timestamps(&self) -> Float64Array {
        Float64Array::from(&self.timestamps[..])
    }

    pub fn get_x(&self) -> Float64Array {
        Float64Array::from(&self.x[..])
    }

    pub fn get_y(&self) -> Float64Array {
        Float64Array::from(&self.y[..])
    }
}

struct Curve {
    timestamps: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
    included: Vec<usize>,
}

impl Curve {
    fn prune(&self) -> Result<DecimatedCurve, String> {
        let mut decimated_curve = DecimatedCurve { timestamps: Vec::new(), x: Vec::new(), y: Vec::new() };

        for index in &self.included {
            let index = index.clone();
            decimated_curve.timestamps.push(get(&self.timestamps, index)?);
            decimated_curve.x.push(get(&self.x, index)?);
            decimated_curve.y.push(get(&self.y, index)?);
        }

        Ok(decimated_curve)
    }
}

#[wasm_bindgen]
pub fn decimate(timestamps: Vec<f64>, x: Vec<f64>, y: Vec<f64>, count: usize) -> Result<DecimatedCurve, String> {
    if timestamps.len() < count {
        return Err("The curve does not have enough points.".to_string());
    }

    if (timestamps.len() != x.len()) || (timestamps.len() != y.len()) {
        return Err("The number of values for timestamps, x-coordinates and y-coordinates don't match.".to_string());
    }
    
    let mut curve = Curve { timestamps, x, y, included: Vec::new() };
    curve.included.push(0);
    curve.included.push(curve.timestamps.len() - 1);
    
    loop {
        let mut add_to_include: Vec<usize> = Vec::new();

        for i in 0..(curve.included.len() - 1) {
            let start = curve.included[i];
            let end = curve.included[i+1];

            if (end - start) <= 1 {
                continue;
            }

            let x1 = curve.x[start];
            let xn = curve.x[end];
            let y1 = curve.y[start];
            let yn = curve.y[end];

            let mut max_distance = perpendicular_distance(curve.x[start+1], curve.y[start+1], x1, y1, xn, yn);
            let mut max_at = start + 1;

            for j in (start+2)..end {
                let d = perpendicular_distance(curve.x[j], curve.y[j], x1, y1, xn, yn);

                if d > max_distance {
                    max_distance = d;
                    max_at = j;
                }
            }

            add_to_include.push(max_at);

            if add_to_include.len() + curve.included.len() == count {
                break;
            }
        }

        curve.included.extend(add_to_include);
        curve.included.sort();

        if curve.included.len() == count {
            break;
        } else if curve.included.len() > count {
            // This should not have happened.
            return Err("The curve could not be decimated properly.".to_string());
        }
    }

    let result = curve.prune()?;
    Ok(result)
}

/// perpendicular distance between a point and a line defined by two points
/// https://math.stackexchange.com/a/2757330
fn perpendicular_distance(x: f64, y: f64, x1: f64, y1: f64, xn: f64, yn: f64) -> f64 {
    let numerator = ((xn - x1)*(y - y1) - (yn - y1)*(x - x1)).abs();
    let denominator = ((xn - x1).powi(2) + (yn - y1).powi(2)).sqrt();

    numerator / denominator
}

fn get<T: Clone>(vector: &Vec<T>, index: usize) -> Result<T, String> {
    if let Some(value) = vector.get(index) {
        Ok(value.clone())
    } else {
        Err("Vector size mismatch: The number of points in the curve did not match each other.".to_string())
    }
}