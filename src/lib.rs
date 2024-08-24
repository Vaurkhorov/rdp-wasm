use js_sys::{Float64Array, JsString};
use wasm_bindgen::prelude::*;

const MAX_BINARY_SEARCH_ITERATIONS: usize = 500;

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
    let mut middle: f64;
    let mut curve = Curve {
        timestamps: Vec::new(),
        x: Vec::new(),
        y: Vec::new(),
    };

    // The loop may hit the limit if two values are somehow removed at the same(or almost the same) tolerance value.
    for _ in 0..MAX_BINARY_SEARCH_ITERATIONS {
        middle = (upper_limit + lower_limit) / 2.0;
        curve = decimate_by_tolerance(timestamps, x, y, middle)?;

        match curve.timestamps.len().cmp(&count) {
            std::cmp::Ordering::Equal => return Ok(curve),
            std::cmp::Ordering::Greater => lower_limit = middle,
            std::cmp::Ordering::Less => upper_limit = middle,
        }
    }

    Err(format!(
        "Binary Search limit reached. Count: {} Middle: {}",
        curve.timestamps.len(),
        (upper_limit + lower_limit) / 2.0
    ))
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
            "Binary Search limit reached. Count: 2 Middle: 0.7071067811865475".to_string()
        );
    }

    #[test]
    fn random_tolerance() {
        let timestamps: Vec<f64> = vec![
            0, 8, 16, 24, 33, 41, 48, 57, 64, 72, 80, 89, 97, 105, 112, 120, 128, 137, 145, 153,
            161, 168, 176, 184, 193, 201, 208, 217, 224, 232, 240, 250, 256, 264, 272, 280, 288,
            297, 305, 312, 320, 328, 336, 344, 353, 360, 368, 376, 384, 392, 401, 409, 416, 424,
            433, 440, 448, 457, 465, 472, 480, 488, 496, 505, 513, 521, 528, 536, 544, 552, 560,
            569, 576, 584, 592, 600, 648,
        ]
        .iter()
        .map(|&x| x as f64)
        .collect();
        let x: Vec<f64> = vec![
            77, 86, 100, 115, 143, 173, 209, 255, 304, 358, 412, 461, 499, 527, 552, 569, 584, 599,
            609, 621, 636, 648, 660, 674, 684, 699, 714, 728, 741, 756, 768, 781, 791, 797, 806,
            814, 821, 826, 832, 837, 844, 849, 856, 860, 864, 869, 876, 880, 886, 891, 895, 896,
            899, 901, 903, 904, 905, 907, 908, 908, 909, 912, 912, 914, 914, 915, 916, 916, 916,
            917, 917, 917, 919, 919, 919, 919, 919,
        ]
        .iter()
        .map(|&x| x as f64)
        .collect();
        let y: Vec<f64> = vec![
            54, 62, 74, 88, 111, 134, 162, 192, 229, 264, 304, 336, 364, 386, 404, 420, 432, 444,
            455, 467, 484, 499, 513, 530, 543, 560, 577, 592, 608, 623, 634, 645, 654, 661, 669,
            674, 680, 688, 693, 698, 703, 708, 715, 718, 724, 728, 733, 738, 741, 745, 748, 750,
            752, 756, 757, 760, 762, 765, 767, 768, 772, 776, 780, 784, 786, 788, 791, 792, 795,
            796, 798, 800, 801, 802, 803, 804, 805,
        ]
        .iter()
        .map(|&x| x as f64)
        .collect();
        let count = 13;

        let result = decimate_to_count(&timestamps, &x, &y, count).unwrap();

        let expected_curve = Curve {
            timestamps: vec![0, 33, 48, 89, 112, 137, 224, 288, 297, 353, 416, 488, 648]
                .iter()
                .map(|&x| x as f64)
                .collect(),
            x: vec![
                77, 143, 209, 461, 552, 599, 741, 821, 826, 864, 899, 912, 919,
            ]
            .iter()
            .map(|&x| x as f64)
            .collect(),
            y: vec![
                54, 111, 162, 336, 404, 444, 608, 680, 688, 724, 752, 776, 805,
            ]
            .iter()
            .map(|&x| x as f64)
            .collect(),
        };

        assert_curve(&result, &expected_curve);
    }
}
