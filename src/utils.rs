#[derive(Clone, Copy, Debug)]
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}

#[derive(Debug)]
pub struct ViewportTransformOptions {
    pub window_pos: cgmath::Vector2<f64>,
    /// Min and max values of x axis on window space
    pub xw: MinMax<f64>,
    /// Min and max values of y axis on window space
    pub yw: MinMax<f64>,
}

/// Window to viewport transformation
pub fn normalize_window_coordinates(options: &ViewportTransformOptions) -> cgmath::Vector2<f32> {
    let ViewportTransformOptions { window_pos, xw, yw } = options;
    let xw_val = window_pos.x;
    let yw_val = window_pos.y;
    // Viewport axes
    let xv = MinMax::<f64> {
        min: -1.0,
        max: 1.0,
    };
    let yv = MinMax::<f64> {
        min: -1.0,
        max: 1.0,
    };

    // Scaling factors
    let sx = (xv.max - xv.min) / (xw.max - xw.min);
    let sy = (yv.max - yv.min) / (yw.max - yw.min);
    let x = (xv.min + (xw_val - xw.min) * sx) as f32;
    let y = (yv.min + (yw_val - yw.min) * sy) as f32;

    cgmath::Vector2::new(x, y)
}
