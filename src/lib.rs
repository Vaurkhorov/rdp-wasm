use js_sys::Float64Array;
use wasm_bindgen::prelude::*;

const MAX_BINARY_SEARCH_ITERATIONS: usize = 100;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Curve {
    timestamps: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
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
}

#[wasm_bindgen]
pub fn decimate_by_tolerance(
    timestamps: &[f64],
    x: &[f64],
    y: &[f64],
    tolerance: f64,
) -> Result<Curve, String> {
    if (timestamps.len() != x.len()) || (timestamps.len() != y.len()) {
        return Err(
            "The number of values for timestamps, x-coordinates and y-coordinates don't match."
                .to_string(),
        );
    }

    let mut decimated_curve = Curve {
        timestamps: vec![timestamps[0], timestamps[timestamps.len() - 1]],
        x: vec![x[0], x[x.len() - 1]],
        y: vec![y[0], y[y.len() - 1]],
    };
    let mut stack: Vec<(usize, usize)> = vec![(0, timestamps.len() - 1)];

    while let Some((start, end)) = stack.pop() {
        let mut dmax = perpendicular_distance(
            x[start + 1],
            y[start + 1],
            x[start],
            y[start],
            x[end],
            y[end],
        );
        let mut dmax_is_at = start + 1;

        for i in (start + 2)..(end) {
            let d = perpendicular_distance(x[i], y[i], x[start], y[start], x[end], y[end]);

            if d > dmax {
                dmax = d;
                dmax_is_at = i;
            }
        }

        if dmax > tolerance {
            // find the end value's index in the final curve and insert this element just before it
            let index = decimated_curve
                .timestamps
                .iter()
                .position(|&i| i == timestamps[end])
                .expect("The 'end' value should already exist in the final curve.");
            decimated_curve
                .timestamps
                .insert(index, timestamps[dmax_is_at]);
            decimated_curve.x.insert(index, x[dmax_is_at]);
            decimated_curve.y.insert(index, y[dmax_is_at]);

            stack.push((start, dmax_is_at));
            stack.push((dmax_is_at, end));
        }
    }

    Ok(decimated_curve)
}

#[wasm_bindgen]
pub fn decimate_to_count(
    timestamps: &[f64],
    x: &[f64],
    y: &[f64],
    count: usize,
) -> Result<Curve, String> {
    if timestamps.len() < count {
        return Err("The curve does not have enough points.".to_string());
    }

    if (timestamps.len() != x.len()) || (timestamps.len() != y.len()) {
        return Err(
            "The number of values for timestamps, x-coordinates and y-coordinates don't match."
                .to_string(),
        );
    }

    let mut max_distance = distance(x[0], y[0], x[1], y[1]);

    for i in 1..(x.len() - 1) {
        if distance(x[i], y[i], x[i + 1], y[i + 1]) > max_distance {
            max_distance = distance(x[i], y[i], x[i + 1], y[i + 1]);
        }
    }

    let mut lower_limit = 0.0;
    let mut upper_limit = max_distance;

    // The loop may hit the limit if two values are somehow removed at the same(or almost the same) tolerance value.
    for _ in 0..MAX_BINARY_SEARCH_ITERATIONS {
        let middle = (upper_limit - lower_limit) / 2.0;
        let curve = decimate_by_tolerance(timestamps, x, y, middle)?;

        match curve.timestamps.len().cmp(&count) {
            std::cmp::Ordering::Equal => return Ok(curve),
            std::cmp::Ordering::Greater => lower_limit = middle,
            std::cmp::Ordering::Less => upper_limit = middle,
        }
    }

    Err("Binary Search limit reached.".to_string())
}

/// perpendicular distance between a point and a line defined by two points
/// https://math.stackexchange.com/a/2757330
fn perpendicular_distance(x: f64, y: f64, x1: f64, y1: f64, xn: f64, yn: f64) -> f64 {
    let numerator = ((xn - x1) * (y - y1) - (yn - y1) * (x - x1)).abs();
    let denominator = ((xn - x1).powi(2) + (yn - y1).powi(2)).sqrt();

    numerator / denominator
}

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perpendicular_distance_on_the_line() {
        let x = 2.0;
        let y = 2.0;
        let x1 = 0.0;
        let y1 = 0.0;
        let xn = 4.0;
        let yn = 4.0;

        let result = perpendicular_distance(x, y, x1, y1, xn, yn);
        assert!(
            (result - 0.0).abs() < 1e-10,
            "Expected distance to be 0, got {}",
            result
        );
    }

    #[test]
    fn test_perpendicular_distance_off_the_line() {
        let x = 5.0;
        let y = 11.0;
        let x1 = 0.0;
        let y1 = 0.0;
        let xn = 20.0;
        let yn = 20.0;

        let result = perpendicular_distance(x, y, x1, y1, xn, yn);
        assert!(
            (result - (3.0 * (2.0_f64).sqrt())).abs() < 1e-10,
            "Expected distance to be approximately {}, got {}",
            3.0 / (2.0_f64).sqrt(),
            result
        );
    }

    fn assert_curve(actual: &Curve, expected: &Curve) {
        assert_eq!(actual.timestamps, expected.timestamps);
        assert_eq!(actual.x, expected.x);
        assert_eq!(actual.y, expected.y);
    }

    #[test]
    fn all_points_pruned() {
        let timestamps = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        let tolerance = 100.0;

        let result = decimate_by_tolerance(&timestamps, &x, &y, tolerance).unwrap();

        let expected_curve = Curve {
            timestamps: vec![0.0, 4.0], // Only the start and end points are retained
            x: vec![0.0, 4.0],
            y: vec![0.0, 2.0],
        };

        assert_curve(&result, &expected_curve);
    }

    #[test]
    fn no_points_pruned() {
        let timestamps = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let x = vec![0.0, 1.9, 4.0, 5.0, 4.0];
        let y = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        let tolerance = 0.0;

        let result = decimate_by_tolerance(&timestamps, &x, &y, tolerance).unwrap();

        let expected_curve = Curve { timestamps, x, y };

        assert_curve(&result, &expected_curve);
    }

    #[test]
    fn specific_count() {
        let timestamps = vec![0.0, 1.0, 2.0, 3.0, 3.5, 4.0];
        let x = vec![0.0, 2.0, 67.2, 5.1, 6.0, 6.4];
        let y = vec![0.0, 0.7, 1.0, 1.4, 2.2, 3.0];

        let result = decimate_to_count(&timestamps, &x, &y, 3).unwrap();

        let expected_curve = Curve {
            timestamps: vec![0.0, 2.0, 4.0],
            x: vec![0.0, 67.2, 6.4],
            y: vec![0.0, 1.0, 3.0],
        };

        assert_curve(&result, &expected_curve);
    }

    #[test]
    fn limit_reached() {
        let timestamps = vec![0.0, 1.0, 2.0, 3.0];
        let x = vec![0.0, 0.0, 1.0, 4.0];
        let y = vec![0.0, 1.0, 0.0, 4.0];

        let result = decimate_to_count(&timestamps, &x, &y, 3);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Binary Search limit reached.".to_string()
        );
    }
}
