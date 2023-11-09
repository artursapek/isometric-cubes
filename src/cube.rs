use crate::grid::Grid;
use geo::algorithm::LineIntersection;
use geo::{polygon, Contains, Coord, Line, Polygon, Rect};
use std::cmp::Ordering;
use wasm_bindgen::prelude::JsValue;

const CUBE_COLOR_LEFT: &str = "hsl(213deg, 20%, 81%)";
const CUBE_COLOR_RIGHT: &str = "hsl(213deg, 20%, 72%)";
const CUBE_COLOR_TOP: &str = "hsl(213deg, 20%, 93%)";
const CUBE_COLOR_STROKE: &str = "hsl(213deg, 20%, 55%)";

#[derive(Debug, Clone, Copy)]
pub struct Cube {
    pub id: usize,

    character: char,
    pub coord: Coord,
    pub size: f64,
    pub bounds: Rect, // cache

    // Mouse interaction state
    pub is_active: bool,
}

pub struct Corners {
    pub tl: Coord,
    pub tr: Coord,
    pub br: Coord,
    pub bl: Coord,
}

impl Cube {
    pub fn new(character: char, x: f64, y: f64, size: f64) -> Self {
        Cube {
            id: 0,
            character,
            coord: Coord { x, y },
            size,
            is_active: false,
            bounds: Rect::new(
                Coord { x, y },
                Coord {
                    x: x + size,
                    y: y + size,
                },
            ),
        }
    }

    #[allow(unused_must_use)]
    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d, grid: &Grid) {
        context.begin_path();

        let size = self.size;

        context.set_line_width(2.0);
        context.set_line_join("round");
        context.set_stroke_style(&JsValue::from_str(CUBE_COLOR_STROKE));

        if self.is_active {
            context.set_line_width(4.0);
            context.set_stroke_style(&JsValue::from_str("#0000ff"));
        }

        // Draw left face
        {
            let a = grid.project(self.coord.x + size, self.coord.y + size, 0.0);
            let b = grid.project(self.coord.x + size, self.coord.y + size, size);
            let c = grid.project(self.coord.x, self.coord.y + size, size);
            let d = grid.project(self.coord.x, self.coord.y + size, 0.0);

            context.begin_path();
            context.move_to(a.0, a.1);
            context.line_to(b.0, b.1);
            context.line_to(c.0, c.1);
            context.line_to(d.0, d.1);
            context.line_to(a.0, a.1);
            context.set_fill_style(&JsValue::from_str(CUBE_COLOR_LEFT));
            context.fill();
            context.stroke();
        }

        {
            // Draw right
            let a = grid.project(self.coord.x + size, self.coord.y + size, 0.0);
            let b = grid.project(self.coord.x + size, self.coord.y + size, size);
            let c = grid.project(self.coord.x + size, self.coord.y, size);
            let d = grid.project(self.coord.x + size, self.coord.y, 0.0);
            context.begin_path();
            context.move_to(a.0, a.1);
            context.line_to(b.0, b.1);
            context.line_to(c.0, c.1);
            context.line_to(d.0, d.1);
            context.line_to(a.0, a.1);
            context.set_fill_style(&JsValue::from_str(CUBE_COLOR_RIGHT));
            context.fill();
            context.stroke();
        }

        // Draw top
        {
            let a = grid.project(self.coord.x, self.coord.y + size, size);
            let b = grid.project(self.coord.x + size, self.coord.y + size, size);
            let c = grid.project(self.coord.x + size, self.coord.y, size);
            let d = grid.project(self.coord.x, self.coord.y, size);
            context.begin_path();
            context.move_to(a.0, a.1);
            context.line_to(b.0, b.1);
            context.line_to(c.0, c.1);
            context.line_to(d.0, d.1);
            context.line_to(a.0, a.1);
            context.set_fill_style(&JsValue::from_str(CUBE_COLOR_TOP));
            context.fill();
            context.stroke();
        }

        {
            let (tx, ty) = grid.project(self.coord.x + size / 2.0, self.coord.y + size / 2.0, size);
            context.translate(tx, ty);
            context.scale(1.0, 0.5);
            context.rotate(-std::f64::consts::PI / 4.0);
            context.set_font(&format!("{}px sans-serif", self.size));
            context.set_text_align("center");
            context.set_text_baseline("middle");
            context.set_fill_style(&JsValue::from_str("#294252"));
            context.fill_text(&self.character.to_string(), 0.0, 0.0);
            //context.fill_text(&format!("{}", self.id), 0.0, 0.0);
        }

        context.reset_transform();
    }

    pub fn shadow(&self, grid: &Grid) -> Option<Polygon> {
        let corners = self.corners();

        let extrapolate = |line: Line| {
            let x_delta = line.end.x - line.start.x;

            let end = Coord {
                x: line.start.x + x_delta * 100.0,
                y: line.start.y + (x_delta * line.slope()) * 100.0,
            };

            Line::new(line.start, end)
        };

        // gl = ground line
        let g_light = grid.project_coord(grid.light_source.x, grid.light_source.y, 0.0);
        let gl_tl: Line = extrapolate(Line::new(
            g_light,
            grid.project_coord(corners.tl.x, corners.tl.y, 0.0),
        ));
        let gl_tr: Line = extrapolate(Line::new(
            g_light,
            grid.project_coord(corners.tr.x, corners.tr.y, 0.0),
        ));
        let gl_br: Line = extrapolate(Line::new(
            g_light,
            grid.project_coord(corners.br.x, corners.br.y, 0.0),
        ));
        let gl_bl: Line = extrapolate(Line::new(
            g_light,
            grid.project_coord(corners.bl.x, corners.bl.y, 0.0),
        ));

        let a_light = grid.project_coord(
            grid.light_source.x,
            grid.light_source.y,
            grid.light_source.z,
        );
        let al_tl: Line = extrapolate(Line::new(
            a_light,
            grid.project_coord(corners.tl.x, corners.tl.y, self.size),
        ));
        let al_tr: Line = extrapolate(Line::new(
            a_light,
            grid.project_coord(corners.tr.x, corners.tr.y, self.size),
        ));
        let al_br: Line = extrapolate(Line::new(
            a_light,
            grid.project_coord(corners.br.x, corners.br.y, self.size),
        ));
        let al_bl: Line = extrapolate(Line::new(
            a_light,
            grid.project_coord(corners.bl.x, corners.bl.y, self.size),
        ));

        let ix_tl = geo::algorithm::line_intersection::line_intersection(gl_tl, al_tl);
        let ix_tr = geo::algorithm::line_intersection::line_intersection(gl_tr, al_tr);
        let ix_bl = geo::algorithm::line_intersection::line_intersection(gl_bl, al_bl);
        let ix_br = geo::algorithm::line_intersection::line_intersection(gl_br, al_br);

        match (ix_tl, ix_tr, ix_bl, ix_br) {
            (
                Some(LineIntersection::SinglePoint {
                    intersection: ix_tl,
                    ..
                }),
                Some(LineIntersection::SinglePoint {
                    intersection: ix_tr,
                    ..
                }),
                Some(LineIntersection::SinglePoint {
                    intersection: ix_bl,
                    ..
                }),
                Some(LineIntersection::SinglePoint {
                    intersection: ix_br,
                    ..
                }),
            ) => {
                let tl = grid.project_coord(corners.tl.x, corners.tl.y, 0.0);
                let tr = grid.project_coord(corners.tr.x, corners.tr.y, 0.0);
                let bl = grid.project_coord(corners.bl.x, corners.bl.y, 0.0);
                let br = grid.project_coord(corners.br.x, corners.br.y, 0.0);
                let x_range = self.coord.x..(self.coord.x + self.size);
                let y_range = self.coord.y..(self.coord.y + self.size);

                let ls = &grid.light_source;

                let center = self.bounds.center();

                match (x_range.contains(&ls.x), y_range.contains(&ls.y)) {
                    (true, true) => Some(polygon![ix_tl, ix_tr, ix_br, ix_bl]),
                    (true, false) => {
                        if center.y > ls.y {
                            Some(polygon![tr, ix_tr, ix_br, ix_bl, ix_tl, tl])
                        } else {
                            Some(polygon![br, ix_br, ix_tr, ix_tl, ix_bl, bl])
                        }
                    }
                    (false, true) => {
                        if center.x > ls.x {
                            Some(polygon![tl, ix_tl, ix_tr, ix_br, ix_bl, bl])
                        } else {
                            Some(polygon![tr, ix_tr, ix_tl, ix_bl, ix_br, br])
                        }
                    }
                    (false, false) => {
                        match (
                            center.x.partial_cmp(&ls.x).unwrap(),
                            center.y.partial_cmp(&ls.y).unwrap(),
                        ) {
                            (Ordering::Greater, Ordering::Greater) => {
                                Some(polygon![tr, ix_tr, ix_br, ix_bl, bl])
                            }
                            (Ordering::Greater, Ordering::Less) => {
                                Some(polygon![br, ix_br, ix_tr, ix_tl, tl])
                            }
                            (Ordering::Less, Ordering::Less) => {
                                Some(polygon![tr, ix_tr, ix_tl, ix_bl, bl])
                            }
                            (Ordering::Less, Ordering::Greater) => {
                                Some(polygon![br, ix_br, ix_bl, ix_tl, tl])
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            (_tl, _tr, _bl, _br) => {
                // TODO handle collinear intersections (rare)
                None
            }
        }
    }

    pub fn hit_test(&self, posn: &Coord, grid: &Grid) -> bool {
        let (x, y) = grid.project(self.coord.x, self.coord.y, 0.0);

        let edges = polygon![
            (x: x, y: y - self.size),
            (x: x + self.size, y: y - self.size / 2.0),
            (x: x + self.size, y: y + self.size / 2.0),
            (x: x, y: y + self.size),
            (x: x - self.size, y: y + self.size / 2.0),
            (x: x - self.size, y: y - self.size / 2.0),
        ];

        edges.contains(&Coord {
            x: posn.x * grid.device_pixel_ratio,
            y: posn.y * grid.device_pixel_ratio,
        })
    }

    pub fn corners(&self) -> Corners {
        let min = self.bounds.min();
        let max = self.bounds.max();

        Corners {
            // TL
            tl: Coord { x: min.x, y: min.y },
            // TR
            tr: Coord { x: max.x, y: min.y },
            // BR
            br: Coord { x: max.x, y: max.y },
            // BL
            bl: Coord { x: min.x, y: max.y },
        }
    }

    pub fn translate(&mut self, x_delta: f64, y_delta: f64) {
        self.coord.x += x_delta;
        self.coord.y += y_delta;
        self.bounds = Rect::new(
            Coord {
                x: self.coord.x,
                y: self.coord.y,
            },
            Coord {
                x: self.coord.x + self.size,
                y: self.coord.y + self.size,
            },
        );
    }

    pub fn translated(mut self, x_delta: f64, y_delta: f64) -> Self {
        self.translate(x_delta, y_delta);
        self
    }

    pub fn overlap(&self, other: &Cube) -> (f64, f64) {
        let x_overlap = if self.bounds.max().x > other.coord.x && self.coord.x <= other.coord.x {
            self.bounds.max().x - other.coord.x
        } else if other.bounds.max().x > self.coord.x && other.coord.x <= self.coord.x {
            other.bounds.max().x - self.coord.x
        } else {
            0.0
        };

        let y_overlap = if self.bounds.max().y > other.coord.y && self.coord.y <= other.coord.y {
            self.bounds.max().y - other.coord.y
        } else if other.bounds.max().y > self.coord.y && other.coord.y <= self.coord.y {
            other.bounds.max().y - self.coord.y
        } else {
            0.0
        };

        (x_overlap, y_overlap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_overlap() {
        let a = Cube::new('A', 0.0, 0.0, 100.0);
        let b = Cube::new('B', 5.0, 7.0, 100.0);
        assert_eq!(a.overlap(&b), (95.0, 93.));

        let a = Cube::new('A', 100.0, 0.0, 100.0);
        let b = Cube::new('B', 5.0, 5.0, 100.0);
        assert_eq!(a.overlap(&b), (5.0, 95.));

        let a = Cube::new('A', 100.0, 100.0, 100.0);
        let b = Cube::new('B', 5.0, 5.0, 100.0);
        assert_eq!(a.overlap(&b), (5.0, 5.0));

        let a = Cube::new('A', 0.0, 100.0, 100.0);
        let b = Cube::new('B', 0.0, 5.0, 100.0);
        assert_eq!(a.overlap(&b), (100.0, 5.0));

        let a = Cube::new('A', 200.0, 200.0, 100.0);
        let b = Cube::new('B', 5.0, 5.0, 100.0);
        assert_eq!(a.overlap(&b), (0.0, 0.0));
    }
}
