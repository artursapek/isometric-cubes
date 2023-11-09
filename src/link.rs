use crate::grid::Grid;
use geo::{coord, polygon, Contains, Coord};
use wasm_bindgen::prelude::*;

pub struct Link {
    coord: Coord,
    text: &'static str,
    pub url: &'static str,

    width: f64,

    pub is_active: bool,
}

const FONT_STYLE: &str = "48px sans-serif";

impl Link {
    pub fn new(
        x: f64,
        y: f64,
        text: &'static str,
        url: &'static str,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Self {
        context.set_font(FONT_STYLE);
        let width = context.measure_text(text).unwrap().width();

        Self {
            coord: coord!(x: x, y: y),
            text,
            url,
            width,
            is_active: false,
        }
    }
    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d, grid: &Grid) {
        let (x, y) = grid.project(self.coord.x, self.coord.y, 0.0);

        let _ = context.translate(x, y);
        let _ = context.scale(1.0, 0.5);
        let _ = context.rotate(-std::f64::consts::PI / 4.0);
        context.set_font(FONT_STYLE);
        context.set_text_align("left");
        context.set_text_baseline("middle");
        context.set_fill_style(&JsValue::from_str("#0000ff"));
        let _ = context.set_global_composite_operation("multiply");
        let _ = context.fill_text(self.text, 0.0, 0.0);
        let _ = context.set_global_composite_operation("source-over");

        if self.is_active {
            let (underline_x, underline_y) = (0.0, 25.0);
            let (underline_x2, underline_y2) = (self.width, 25.0);
            context.begin_path();
            context.move_to(underline_x, underline_y);
            context.line_to(underline_x2, underline_y2);
            context.set_line_width(4.0);
            context.set_stroke_style(&JsValue::from_str("#0000ff"));
            context.stroke();
        }

        let _ = context.reset_transform();
    }

    pub fn hit_test(&self, posn: &Coord, grid: &Grid) -> bool {
        let (x, y) = grid.project(self.coord.x - 20.0, self.coord.y + 10.0, 0.0);
        let (x2, y2) = grid.project(self.coord.x - 20.0, self.coord.y - self.width * 0.75, 0.0);
        let (x3, y3) = grid.project(self.coord.x + 20.0, self.coord.y - self.width * 0.75, 0.0);
        let (x4, y4) = grid.project(self.coord.x + 20.0, self.coord.y + 10.0, 0.0);

        let edges = polygon![
            (x: x, y: y),
            (x: x2, y: y2),
            (x: x3, y: y3),
            (x: x4, y: y4),
        ];

        edges.contains(&Coord {
            x: posn.x * grid.device_pixel_ratio,
            y: posn.y * grid.device_pixel_ratio,
        })
    }
}
