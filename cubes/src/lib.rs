use wasm_bindgen::prelude::*;

mod cube;
mod cube_grid;
mod grid;
mod link;
mod movement;

use cube::Cube;
use cube_grid::CubeGrid;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[allow(unused)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    document.body().unwrap().append_child(&canvas)?;

    let mut cube_grid = CubeGrid::new(canvas);

    cube_grid.insert_cubes(vec![
        Cube::new('A', 7.0, 450.0, 100.0),
        Cube::new('R', -5.0, 325.0, 100.0),
        Cube::new('T', -12.0, 213.0, 100.0),
        Cube::new('U', 8.0, 105.0, 100.0),
        Cube::new('R', 3.0, 0.0, 100.0),
        Cube::new('S', 145.0, 430.0, 100.0),
        Cube::new('A', 120.0, 320.0, 100.0),
        Cube::new('P', 143.0, 197.0, 100.0),
        Cube::new('E', 140.0, 80.0, 100.0),
        Cube::new('K', 130.0, -40.0, 100.0),
        Cube::new('S', 360.0, 600.0, 60.0),
        Cube::new('O', 365.0, 530.0, 60.0),
        Cube::new('F', 357.0, 457.0, 60.0),
        Cube::new('T', 360.0, 391.0, 60.0),
        Cube::new('W', 356.0, 316.0, 60.0),
        Cube::new('A', 354.0, 246.0, 60.0),
        Cube::new('R', 360.0, 176.0, 60.0),
        Cube::new('E', 360.0, 112.0, 60.0),
        Cube::new('D', 430.0, 570.0, 60.0),
        Cube::new('E', 430.0, 500.0, 60.0),
        Cube::new('V', 430.0, 427.0, 60.0),
        Cube::new('E', 434.0, 363.0, 60.0),
        Cube::new('L', 430.0, 293.0, 60.0),
        Cube::new('O', 530.0, 217.0, 60.0),
        Cube::new('P', 430.0, 149.0, 60.0),
        Cube::new('E', 430.0, 82.0, 60.0),
        Cube::new('R', 456.0, -5.0, 60.0),
    ]);

    cube_grid.start()
}
