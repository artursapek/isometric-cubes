use geo::{polygon, Intersects, Polygon};
use std::cmp::Ordering;

use crate::Cube;

#[derive(Debug, PartialEq)]
enum Direction {
    None,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Direction {
    fn from_deltas(dx: f64, dy: f64) -> Self {
        match (dx.partial_cmp(&0.0), dy.partial_cmp(&0.0)) {
            (Some(Ordering::Equal), Some(Ordering::Less)) => Self::Up,
            (Some(Ordering::Greater), Some(Ordering::Less)) => Self::UpRight,
            (Some(Ordering::Greater), Some(Ordering::Equal)) => Self::Right,
            (Some(Ordering::Greater), Some(Ordering::Greater)) => Self::DownRight,
            (Some(Ordering::Equal), Some(Ordering::Greater)) => Self::Down,
            (Some(Ordering::Less), Some(Ordering::Greater)) => Self::DownLeft,
            (Some(Ordering::Less), Some(Ordering::Equal)) => Self::Left,
            (Some(Ordering::Less), Some(Ordering::Less)) => Self::UpLeft,
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Self::None,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Movement {
    pub start: Cube,
    pub end: Cube,

    direction: Direction,
    pub dx: f64,
    pub dy: f64,
    poly: Polygon,
}

// Takes a cube and deltas and produces a Movement
// which contains said deltas as well as a polygon representing
// the area covered by this movement.
//
// _____ . . .
// |   |      . . . .
// |___|. . .        . . .____
//            . . .      |   |
//                 . . . |___|
//
impl Movement {
    pub fn new(start: Cube, end: Cube) -> Self {
        if start.size != end.size {
            panic!("Can't generate movement for a cube changing size");
        }

        let start_corners = start.corners();
        let end_corners = end.corners();

        let dx = end.coord.x - start.coord.x;
        let dy = end.coord.y - start.coord.y;

        // 8 possible cases here:
        // 4 cases for non-zero x delta and y delta
        // 4 cases where one of the deltas is zero

        let direction = Direction::from_deltas(dx, dy);

        let poly = match direction {
            Direction::UpLeft | Direction::DownRight => polygon![
                (x: start_corners.tr.x, y: start_corners.tr.y),
                (x: start_corners.bl.x, y: start_corners.bl.y),
                (x: end_corners.bl.x, y: end_corners.bl.y),
                (x: end_corners.tr.x, y: end_corners.tr.y),
            ],
            Direction::UpRight | Direction::DownLeft => polygon![
                (x: start_corners.tl.x, y: start_corners.tl.y),
                (x: start_corners.br.x, y: start_corners.br.y),
                (x: end_corners.br.x, y: end_corners.br.y),
                (x: end_corners.tl.x, y: end_corners.tl.y),
            ],
            Direction::Up => polygon![
                (x: start_corners.tl.x, y: start_corners.tl.y),
                (x: start_corners.tr.x, y: start_corners.tr.y),
                (x: end_corners.br.x, y: end_corners.br.y),
                (x: end_corners.bl.x, y: end_corners.bl.y),
            ],
            Direction::Down => polygon![
                (x: start_corners.bl.x, y: start_corners.bl.y),
                (x: start_corners.br.x, y: start_corners.br.y),
                (x: end_corners.tr.x, y: end_corners.tr.y),
                (x: end_corners.tl.x, y: end_corners.tl.y),
            ],
            Direction::Left => polygon![
                (x: start_corners.tl.x, y: start_corners.tl.y),
                (x: start_corners.bl.x, y: start_corners.bl.y),
                (x: end_corners.br.x, y: end_corners.br.y),
                (x: end_corners.tr.x, y: end_corners.tr.y),
            ],
            Direction::Right => polygon![
                (x: start_corners.tr.x, y: start_corners.tr.y),
                (x: start_corners.br.x, y: start_corners.br.y),
                (x: end_corners.bl.x, y: end_corners.bl.y),
                (x: end_corners.tl.x, y: end_corners.tl.y),
            ],
            Direction::None => polygon![],
        };

        Movement {
            start,
            end,
            direction,
            dx,
            dy,
            poly,
        }
    }

    pub fn new_from_delta(cube: Cube, dx: f64, dy: f64) -> Self {
        Self::new(cube, cube.translated(dx, dy))
    }

    pub fn impact(&self, subject: Cube) -> Option<Movement> {
        let (overlap_x, overlap_y) = self.end.overlap(&subject);

        if overlap_x > 0.0 && overlap_y > 0.0 {
            // The final position of the cube which moved now overlaps with subject cube
            let center = self.start.bounds.center();
            let subject_center = subject.bounds.center();

            let cx_delta = (center.x - subject_center.x).abs();
            let cy_delta = (center.y - subject_center.y).abs();

            let end = match self.direction {
                Direction::UpLeft => {
                    if cx_delta > cy_delta {
                        // Subject is to our left; move it left
                        subject.translated(-overlap_x, 0.0)
                    } else if cx_delta < cy_delta {
                        // Subject is to our right; move it up
                        subject.translated(0.0, -overlap_y)
                    } else {
                        // Head on!
                        subject.translated(-overlap_x, -overlap_y)
                    }
                }

                Direction::DownRight => {
                    if cx_delta < cy_delta {
                        // Subject is to our left; move it down
                        subject.translated(0.0, overlap_y)
                    } else if cx_delta > cy_delta {
                        // Subject is to our right; move it right
                        subject.translated(overlap_x, 0.0)
                    } else {
                        // Head on!
                        subject.translated(overlap_x, overlap_y)
                    }
                }

                Direction::DownLeft => {
                    if cx_delta > cy_delta {
                        // Subject is above us; move it left
                        subject.translated(-overlap_x, 0.0)
                    } else if cx_delta < cy_delta {
                        // Subject is below us; move it down
                        subject.translated(0.0, overlap_y)
                    } else {
                        // Head on!
                        subject.translated(-overlap_x, overlap_y)
                    }
                }

                Direction::UpRight => {
                    if cx_delta < cy_delta {
                        // Subject is above us; move it up
                        subject.translated(0.0, -overlap_y)
                    } else if cx_delta > cy_delta {
                        // Subject is below us; move it right
                        subject.translated(overlap_x, 0.0)
                    } else {
                        // Head on!
                        subject.translated(overlap_x, -overlap_y)
                    }
                }

                Direction::Left => subject.translated(-overlap_x, 0.0),
                Direction::Right => subject.translated(overlap_x, 0.0),
                Direction::Up => subject.translated(0.0, -overlap_y),
                Direction::Down => subject.translated(0.0, overlap_y),

                Direction::None => subject,
            };

            Some(Movement::new(subject, end))
        } else if self.poly.intersects(&subject.bounds) {
            // The final position of the cube which moved does NOT overlap with subject cube,
            // but it did completely pass through the subject cube on its way there.
            // Here we use the motion sweep polygon `poly` to determine what to do.
            // TODO allowing pass through for now
            None
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_movement_impact() {
        // Basic movement example going down and to the right
        let start = Cube::new('A', 0.0, 0.0, 100.0);
        let end = Cube::new('A', 60.0, 60.0, 100.0);
        let movement = Movement::new(start, end);
        assert_eq!(movement.direction, Direction::DownRight);

        // Moving other cubes...
        // Moving y
        let subject = Cube::new('B', 20.0, 120.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, 0.0);
        assert_eq!(impact.dy, 40.0);

        // Moving x
        let subject = Cube::new('B', 120.0, 20.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, 40.0);
        assert_eq!(impact.dy, 0.0);

        // Moving y when that's the largest overlap, because of a huge y delta
        let start = Cube::new('A', 0.0, 0.0, 100.0);
        let end = Cube::new('A', 10.0, 110.0, 100.0);
        let movement = Movement::new(start, end);
        let subject = Cube::new('B', 90.0, 120.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, 0.0);
        assert_eq!(impact.dy, 90.0);

        // Moving x when that's the largest overlap, because of a huge y delta
        let start = Cube::new('A', 0.0, 0.0, 100.0);
        let end = Cube::new('A', 140.0, 20.0, 100.0);
        let movement = Movement::new(start, end);
        let subject = Cube::new('B', 110.0, 10.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, 70.0);
        assert_eq!(impact.dy, 0.0);

        // Moving x and y when the movement and overlap are truly equal
        let start = Cube::new('A', 0.0, 0.0, 100.0);
        let end = Cube::new('A', 30.0, 30.0, 100.0);
        let movement = Movement::new(start, end);
        let subject = Cube::new('B', 110.0, 110.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, 20.0);
        assert_eq!(impact.dy, 20.0);

        // In reverse :-)
        let start = Cube::new('A', 160.0, 160.0, 100.0);
        let end = Cube::new('A', 130.0, 140.0, 100.0);
        let movement = Movement::new(start, end);
        assert_eq!(movement.direction, Direction::UpLeft);
        assert_eq!(movement.dx, -30.0);
        assert_eq!(movement.dy, -20.0);
        let subject = Cube::new('B', 50.0, 50.0, 100.0);
        let impact = movement.impact(subject).unwrap();
        assert_eq!(impact.dx, -20.0);
        assert_eq!(impact.dy, -10.0);
    }

    fn test_cube_movement_directions() {
        // DownLeft
        let start = Cube::new('A', 0.0, 0.0, 100.0);
        let end = Cube::new('A', -50.0, 50.0, 100.0);
        let movement = Movement::new(start, end);
        assert_eq!(movement.direction, Direction::DownLeft);

        // TODO
    }
}
