use nalgebra::{DMatrix, DVector};
use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct Plane {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

/// Fit a plane ax + by + cz + d = 0 to 3D points using SVD
/// Returns (a, b, c, d) coefficients
pub fn fit_plane(positions: &[Vec<f32>]) -> Option<Plane> {
    if positions.is_empty() {
        return None;
    }

    let n = positions.len();

    let a = DMatrix::from_iterator(n, 3, positions.iter().cloned().flatten().map(|x| x as f64));
    let b = DVector::from_element(n, -1.0);

    // Solve using SVD least squares
    let svd = a.svd(true, true);
    let solution = svd.solve(&b, 1e-10).unwrap();

    Some(Plane {
        a: solution[0],
        b: solution[1],
        c: solution[2],
        d: 1.0, // d = 1 (fixed)
    })
}

/// Calculate plane metrics compared to ground truth y = 20 (i.e., 0x + 1y + 0z - 20 = 0)
pub fn calculate_plane_metrics(plane: Plane) -> PlaneMetrics {
    // Normalize the plane equation
    let norm = (plane.a * plane.a + plane.b * plane.b + plane.c * plane.c).sqrt();
    let (a_norm, b_norm, c_norm, d_norm) = if norm > 1e-10 {
        (
            plane.a / norm,
            plane.b / norm,
            plane.c / norm,
            plane.d / norm,
        )
    } else {
        (plane.a, plane.b, plane.c, plane.d)
    };

    // Ground truth: y = 20 -> 0x + 1y + 0z - 20 = 0
    // Normalized: 0x + 1y + 0z - 20 = 0 (already normalized)
    let gt_a = 0.0;
    let gt_b = 1.0;
    let gt_c = 0.0;
    let gt_d = -20.0;

    // Angular error between normal vectors
    let dot_product = a_norm * gt_a + b_norm * gt_b + c_norm * gt_c;
    let angle_error_rad = dot_product.abs().acos();
    let angle_error_deg = angle_error_rad.to_degrees();

    // Distance error (perpendicular distance between parallel planes)
    // For normalized planes, distance = |d1 - d2| if normals are parallel
    let distance_error = (d_norm - gt_d).abs();

    // Y-intercept when x=0, z=0: by + d = 0 -> y = -d/b
    let y_intercept = if b_norm.abs() > 1e-10 {
        -d_norm / b_norm
    } else {
        f64::NAN
    };
    let y_intercept_error = (y_intercept - 20.0).abs();

    PlaneMetrics {
        a: a_norm,
        b: b_norm,
        c: c_norm,
        d: d_norm,
        angle_error_deg,
        distance_error,
        y_intercept,
        y_intercept_error,
    }
}

#[derive(Debug, Clone)]
pub struct PlaneMetrics {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    angle_error_deg: f64,
    distance_error: f64,
    y_intercept: f64,
    y_intercept_error: f64,
}

/// Analyze the dataframe grouped by angle_step and servo_motor_delay
pub fn analyze_experiment(df: DataFrame) -> PolarsResult<DataFrame> {
    // First, extract the property columns (they should be constant per recording)
    // Get unique angle_step and servo_motor_delay values per group
    let df_with_params = df
        .lazy()
        .with_columns([
            col("property:angle_step:Scalars:scalars")
                .list()
                .first()
                .alias("angle_step"),
            col("property:servo_motor_delay:Scalars:scalars")
                .list()
                .first()
                .alias("servo_motor_delay"),
            col("property:RecordingInfo:start_time")
                .list()
                .first()
                .alias("start_time"),
        ])
        .collect()?;

    // Group by the parameters
    // We'll need to do custom aggregation for plane fitting
    // For that we collect the data we need per group
    let grouped = df_with_params.group_by(["start_time", "angle_step", "servo_motor_delay"])?;

    let dataframe = grouped.apply(process_group);
    dataframe
}

fn process_group(df: DataFrame) -> PolarsResult<DataFrame> {
    let mut angle_steps = Vec::new();
    let mut servo_delays = Vec::new();
    let mut total_times = Vec::new();
    let mut plane_a = Vec::new();
    let mut plane_b = Vec::new();
    let mut plane_c = Vec::new();
    let mut plane_d = Vec::new();
    let mut angle_errors = Vec::new();
    let mut distance_errors = Vec::new();
    let mut y_intercepts = Vec::new();
    let mut y_intercept_errors = Vec::new();
    let mut num_points = Vec::new();

    // Extract angle_step and servo_delay (should be same for all rows in group)
    let angle_step_col = df.column("angle_step")?;
    let servo_delay_col = df.column("servo_motor_delay")?;

    let angle_step_val = angle_step_col
        .f64()?
        .first()
        .ok_or(PolarsError::NoData("No data for angle_step".into()))?;
    let servo_delay_val = servo_delay_col
        .f64()?
        .first()
        .ok_or(PolarsError::NoData("No data for servo_motor_delay".into()))?;

    // Calculate total time in seconds
    let capture_time_col = df.column("capture_time")?;
    let timestamps = capture_time_col
        .as_materialized_series()
        .timestamp(TimeUnit::Nanoseconds)?;
    let min_time = timestamps.min().unwrap_or(0);
    let max_time = timestamps.max().unwrap_or(0);
    let total_time_ns = max_time - min_time;
    let total_time_ms = total_time_ns as f64 / 1_000_000.0;

    // Extract positions for plane fitting
    let positions_col = df.column("/position:Points3D:positions")?;
    let positions_series = positions_col.as_materialized_series();
    let positions_list = positions_series.list()?;

    let mut all_positions = Vec::new();
    for i in 0..positions_list.len() {
        if let Some(series) = positions_list.get_as_series(i) {
            if let Ok(inner_array) = series.array() {
                for j in 0..inner_array.len() {
                    if let Some(point_series) = inner_array.get_as_series(j) {
                        if let Ok(floats) = point_series.f32() {
                            let point: Vec<f32> = floats.into_iter().filter_map(|x| x).collect();
                            if point.len() >= 3 {
                                all_positions.push(point);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fit plane
    let plane = fit_plane(&all_positions).unwrap();
    let metrics = calculate_plane_metrics(plane);

    angle_steps.push(angle_step_val);
    servo_delays.push(servo_delay_val);
    total_times.push(total_time_ms);
    plane_a.push(metrics.a);
    plane_b.push(metrics.b);
    plane_c.push(metrics.c);
    plane_d.push(metrics.d);
    angle_errors.push(metrics.angle_error_deg);
    distance_errors.push(metrics.distance_error);
    y_intercepts.push(metrics.y_intercept);
    y_intercept_errors.push(metrics.y_intercept_error);
    num_points.push(all_positions.len() as f64);

    // Create result dataframe
    DataFrame::new(vec![
        Series::new("angle_step".into(), angle_steps).into(),
        Series::new("servo_motor_delay".into(), servo_delays).into(),
        Series::new("total_time_ms".into(), total_times).into(),
        Series::new("num_points".into(), num_points).into(),
        Series::new("plane_a".into(), plane_a).into(),
        Series::new("plane_b".into(), plane_b).into(),
        Series::new("plane_c".into(), plane_c).into(),
        Series::new("plane_d".into(), plane_d).into(),
        Series::new("angle_error_deg".into(), angle_errors).into(),
        Series::new("distance_error".into(), distance_errors).into(),
        Series::new("y_intercept".into(), y_intercepts).into(),
        Series::new("y_intercept_error".into(), y_intercept_errors).into(),
    ])
}

/// Calculate repeatability statistics for each parameter combination
pub fn calculate_repeatability(results_df: &DataFrame) -> PolarsResult<DataFrame> {
    results_df
        .clone()
        .lazy()
        .group_by([col("angle_step"), col("servo_motor_delay")])
        .agg([
            // Time statistics
            col("total_time_ms").mean().alias("avg_time_ms"),
            col("total_time_ms").std(1).alias("std_time_ms"),
            // Error statistics
            col("angle_error_deg").mean().alias("avg_angle_error_deg"),
            col("angle_error_deg").std(1).alias("std_angle_error_deg"),
            col("y_intercept_error").mean().alias("avg_y_error"),
            col("y_intercept_error").std(1).alias("std_y_error"),
            // Y-intercept statistics
            col("y_intercept").mean().alias("avg_y_intercept"),
            col("y_intercept").std(1).alias("std_y_intercept"),
            // Count repetitions
            col("angle_step").count().alias("num_repetitions"),
            // Point count
            col("num_points").mean().alias("avg_num_points"),
        ])
        .sort(["avg_y_error"], SortMultipleOptions::default())
        .collect()
}
