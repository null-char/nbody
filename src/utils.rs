use uuid::Uuid;

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
    /// Min and max values of x axis on viewport space
    pub xv: MinMax<f64>,
    /// Min and max values of y axis on viewport space
    pub yv: MinMax<f64>,
}

/// Window to viewport transformation
pub fn normalize_window_coordinates(options: &ViewportTransformOptions) -> cgmath::Vector2<f32> {
    let ViewportTransformOptions {
        window_pos,
        xw,
        yw,
        xv,
        yv,
    } = options;
    let xw_val = window_pos.x;
    let yw_val = window_pos.y;

    // Scaling factors
    let sx = (xv.max - xv.min) / (xw.max - xw.min);
    let sy = (yv.max - yv.min) / (yw.max - yw.min);
    let x = (xv.min + (xw_val - xw.min) * sx) as f32;
    let y = (yv.min + (yw_val - yw.min) * sy) as f32;

    cgmath::Vector2::new(x, y)
}

/// Generates a new v4 UUID
pub fn generate_new_uuid() -> Uuid {
    Uuid::new_v4()
}
