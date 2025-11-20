use geo::CoordsIter;
use geo::{Coord, Polygon};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::cube::Cube;
use crate::grid::Grid;
use crate::link::Link;
use crate::movement::Movement;

pub struct CubeGrid {
    cubes: Vec<Cube>,
    grid: Grid,

    links: Vec<Link>,

    next_id: usize,

    canvas: web_sys::HtmlCanvasElement,
    context: Rc<web_sys::CanvasRenderingContext2d>,

    window_dimensions: Rc<RefCell<WindowDimensions>>,

    touch_events: Rc<RefCell<Vec<web_sys::TouchEvent>>>,
    mouse_events: Rc<RefCell<Vec<MouseEvent>>>,
    mouse_state: MouseState,
}

#[derive(Default)]
enum Cursor {
    #[default]
    Default,
    Grab,
    Link,
}

impl Cursor {
    fn to_css_string(&self, is_active: bool) -> &'static str {
        match (self, is_active) {
            (Cursor::Default, _) => "default",
            (Cursor::Grab, false) => "grab",
            (Cursor::Grab, true) => "grabbing",
            (Cursor::Link, _) => "pointer",
        }
    }
}

#[derive(Clone, Copy)]
struct MouseEvent {
    is_pressed: bool,
    position: Coord,
}

#[derive(Default)]
struct WindowDimensions {
    width: f64,
    height: f64,
    device_pixel_ratio: f64,
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn window_dimens() -> (f64, f64, f64) {
    let window = web_sys::window().unwrap();

    let width: f64 = window.inner_width().unwrap().as_f64().unwrap();

    let height: f64 = window.inner_height().unwrap().as_f64().unwrap();

    let device_pixel_ratio: f64 = web_sys::window().unwrap().device_pixel_ratio();

    (width, height, device_pixel_ratio)
}

impl From<web_sys::MouseEvent> for MouseEvent {
    fn from(event: web_sys::MouseEvent) -> Self {
        Self {
            is_pressed: event.buttons() == 1,
            position: Coord {
                x: event.client_x() as f64,
                y: event.client_y() as f64,
            },
        }
    }
}

#[derive(Default)]
struct MouseState {
    last_position: Coord,
    last_down_position: Coord,
    current_position: Coord,

    // When this is running on a touch device, we track
    // the first touch and only use that (no multitouch support for now!)
    current_touch: Option<i32>,

    is_pressed: bool,
    is_dragging_cube: bool,
    just_released: bool,

    cursor_style: Cursor,
}

impl MouseState {
    fn cursor_string(&self) -> &'static str {
        self.cursor_style.to_css_string(self.is_pressed)
    }
}

impl CubeGrid {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        let context = Rc::new(context);

        CubeGrid {
            next_id: 1,

            grid: Grid::default(),

            cubes: Vec::new(),
            links: Vec::new(),

            canvas,
            context,

            // Added to by canvas event listeners, consumed in handle_mouse_events
            mouse_events: Rc::new(RefCell::new(Vec::new())),
            touch_events: Rc::new(RefCell::new(Vec::new())),
            window_dimensions: Rc::new(RefCell::new(WindowDimensions::default())),

            mouse_state: MouseState::default(),
        }
    }

    pub fn insert_cubes(&mut self, cubes: Vec<Cube>) {
        for mut cube in cubes {
            cube.id = self.next_id;
            self.next_id += 1;
            self.cubes.push(cube);
        }
    }

    pub fn insert_link(&mut self, x: f64, y: f64, label: &'static str, url: &'static str) {
        self.links.push(Link::new(
                x,y,label,url,&self.context,
        ))
    }

    #[allow(unused_must_use)]
    pub fn start(mut self) -> Result<(), JsValue> {
        // Handle mouse events
        {
            let mouse_events = self.mouse_events.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                mouse_events.borrow_mut().push(event.into());
            });

            // Mouse events
            self.canvas
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            self.canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            self.canvas
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;

            closure.forget();
        }

        {
            let touch_events = self.touch_events.clone();
            let closure2 = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                touch_events.borrow_mut().push(event);
            });

            // Touch events
            self.canvas.add_event_listener_with_callback(
                "touchstart",
                closure2.as_ref().unchecked_ref(),
            )?;
            self.canvas
                .add_event_listener_with_callback("touchmove", closure2.as_ref().unchecked_ref())?;
            self.canvas
                .add_event_listener_with_callback("touchend", closure2.as_ref().unchecked_ref())?;

            closure2.forget();
        }

        // Handle window resize events
        {
            let window_dimensions = self.window_dimensions.clone();
            let canvas = self.canvas.clone();

            let update_dimensions = move || {
                let (width, height, device_pixel_ratio) = window_dimens();
                canvas.style().set_property("width", &width.to_string());
                canvas.style().set_property("height", &height.to_string());
                canvas.set_width((width * device_pixel_ratio) as u32);
                canvas.set_height((height * device_pixel_ratio) as u32);

                *window_dimensions.borrow_mut() = WindowDimensions {
                    width,
                    height,
                    device_pixel_ratio,
                }
            };

            update_dimensions();

            let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::MouseEvent| {
                update_dimensions();
            });

            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Main render loop
        {
            let f = Rc::new(RefCell::new(None));
            let g = f.clone();
            let context = self.context.clone();

            // Initial render
            self.sort_cubes();
            self.update_dimens();
            self.draw(&context);

            *g.borrow_mut() = Some(Closure::new(move || {
                // Schedule ourself for another requestAnimationFrame callback.
                /*
                let performance = web_sys::window()
                    .unwrap()
                    .performance()
                    .expect("performance should be available");

                let now = performance.now();
                let offset = (now % 3000.0) / 10.0;
                */

                let events_changed = self.handle_input_events();
                let dimens_changed = self.update_dimens();

                if dimens_changed || events_changed {
                    self.handle_mouse_state();
                    self.sort_cubes();
                    self.draw(&context);
                }

                self.canvas
                    .style()
                    .set_property("cursor", self.mouse_state.cursor_string());

                request_animation_frame(f.borrow().as_ref().unwrap());
            }));

            request_animation_frame(g.borrow().as_ref().unwrap());
        }

        Ok(())
    }

    pub fn update_dimens(&mut self) -> bool {
        let dimens = self.window_dimensions.borrow();
        self.grid
            .update_dimensions(dimens.width, dimens.height, dimens.device_pixel_ratio)
    }

    pub fn handle_input_events(&mut self) -> bool {
        let mut mouse_events = self.mouse_events.borrow_mut();
        let mut touch_events = self.touch_events.borrow_mut();

        if !mouse_events.is_empty() {
            self.mouse_state.last_position = self.mouse_state.current_position;

            self.mouse_state.just_released = false;

            for event in mouse_events.iter() {
                self.mouse_state.current_position = event.position;

                if event.is_pressed && !self.mouse_state.is_pressed {
                    self.mouse_state.is_pressed = true;
                    self.mouse_state.last_down_position = event.position;
                } else if !event.is_pressed {
                    // released after click
                    if self.mouse_state.is_pressed {
                        self.mouse_state.just_released = true;
                    }

                    self.mouse_state.is_pressed = false;
                }
            }

            *mouse_events = Vec::new();

            true
        } else if !touch_events.is_empty() {
            self.mouse_state.last_position = self.mouse_state.current_position;

            for event in touch_events.iter() {
                let touches = event.touches();

                match touches.get(0) {
                    Some(touch) => {
                        let id = touch.identifier();
                        if let Some(current_touch) = self.mouse_state.current_touch {
                            if id == current_touch {
                                self.mouse_state.is_pressed = true;
                                self.mouse_state.current_position = Coord {
                                    x: touch.client_x() as f64,
                                    y: touch.client_y() as f64,
                                }
                            } else {
                                // The currently tracked touch disappeared and we're left with
                                // a different one. Don't do anything until we reset to 0
                                // touches.
                            }
                        } else {
                            self.mouse_state.current_touch = Some(id);
                            let mp: Coord = Coord {
                                x: touch.client_x() as f64,
                                y: touch.client_y() as f64,
                            };
                            self.mouse_state.current_position = mp;
                            self.mouse_state.last_down_position = mp;
                            self.mouse_state.is_pressed = true;
                        }
                    }
                    None => {
                        // Touch interaction was released
                        if self.mouse_state.current_touch.is_some() {
                            self.mouse_state.current_touch = None;
                            self.mouse_state.just_released = true;
                            self.mouse_state.is_pressed = false;
                        }
                    }
                }
            }

            *touch_events = Vec::new();

            true
        } else {
            false
        }
    }

    fn sort_cubes(&mut self) {
        self.cubes.sort_by(|a, b| {
            let a = self
                .grid
                .project(a.coord.x + a.size / 2.0, a.coord.y + a.size / 2.0, 0.0);
            let b = self
                .grid
                .project(b.coord.x + b.size / 2.0, b.coord.y + b.size / 2.0, 0.0);
            b.y.partial_cmp(&a.y).unwrap()
        });
    }

    fn handle_mouse_state(&mut self) {
        if self.mouse_state.is_pressed && self.mouse_state.is_dragging_cube {
            // Actively dragging a cube
            let (current_iso_x, current_iso_y) = self.grid.cartesian_to_iso(
                self.mouse_state.current_position.x,
                self.mouse_state.current_position.y,
            );
            let (last_iso_x, last_iso_y) = self.grid.cartesian_to_iso(
                self.mouse_state.last_position.x,
                self.mouse_state.last_position.y,
            );

            let dx = current_iso_x - last_iso_x;
            let dy = current_iso_y - last_iso_y;

            if dx == 0.0 && dy == 0.0 {
                return;
            }

            let mut cubes_moved: HashSet<usize> = HashSet::new();
            let mut cubes_to_move: Vec<Movement> = Vec::new();

            for cube in &mut self.cubes {
                if cube.is_active {
                    cubes_to_move.push(Movement::new_from_delta(*cube, dx, dy));
                }
            }

            while let Some(movement) = cubes_to_move.pop() {
                let mut last_movement: Option<Movement> = None;

                for cube in &mut self.cubes {
                    if cube.id == movement.end.id {
                        cube.translate(movement.dx, movement.dy);

                        cubes_moved.insert(cube.id);

                        last_movement = Some(movement);

                        break;
                    }
                }

                if let Some(last_movement) = last_movement {
                    // check other overlapping cubes
                    for other_cube in &self.cubes {
                        if !cubes_moved.contains(&other_cube.id) {
                            if let Some(impact) = last_movement.impact(*other_cube) {
                                cubes_to_move.push(impact);
                            }
                        }
                    }
                }
            }
        } else if !self.mouse_state.is_pressed && self.mouse_state.is_dragging_cube {
            // Just stopped dragging
            for cube in &mut self.cubes {
                if cube.is_active {
                    self.mouse_state.is_dragging_cube = false;
                }
            }
        } else {
            // Not dragging anything, just moving cursor around
            let mut found_active_object = false;
            self.mouse_state.is_dragging_cube = false;

            for cube in &mut self.cubes {
                if !found_active_object
                    && cube.hit_test(&self.mouse_state.current_position, &self.grid)
                {
                    cube.is_active = true;
                    found_active_object = true;

                    self.mouse_state.cursor_style = Cursor::Grab;

                    if self.mouse_state.is_pressed {
                        self.mouse_state.is_dragging_cube = true;
                    }
                } else {
                    cube.is_active = false;
                }
            }

            for link in &mut self.links {
                if !found_active_object
                    && link.hit_test(&self.mouse_state.current_position, &self.grid)
                {
                    found_active_object = true;

                    self.mouse_state.cursor_style = Cursor::Link;

                    link.is_active = !self.mouse_state.is_pressed;

                    if self.mouse_state.just_released {
                        let _ = web_sys::window()
                            .unwrap()
                            .open_with_url_and_target(link.url, "_top");
                    }
                } else {
                    link.is_active = false;
                }
            }

            if !found_active_object {
                self.mouse_state.cursor_style = Cursor::Default;
            }
        }
    }

    fn draw(&mut self, context: &web_sys::CanvasRenderingContext2d) {
        context.clear_rect(
            0.0,
            0.0,
            self.grid.width * self.grid.device_pixel_ratio,
            self.grid.height * self.grid.device_pixel_ratio,
        );

        self.draw_shadow(context);

        for link in &self.links {
            link.draw(context, &self.grid);
        }

        for cube in self.cubes.iter().rev() {
            cube.draw(context, &self.grid);
        }
    }

    fn draw_shadow(&mut self, context: &web_sys::CanvasRenderingContext2d) {
        let shadows: Vec<Polygon> = self
            .cubes
            .iter()
            .flat_map(|cube| cube.shadow(&self.grid))
            .collect();

        context.set_fill_style(&JsValue::from_str("rgba(245, 245, 245, 1.0)"));

        for shadow in shadows {
            context.begin_path();
            for (i, coord) in shadow.coords_iter().enumerate() {
                if i == 0 {
                    context.move_to(coord.x, coord.y);
                } else {
                    context.line_to(coord.x, coord.y);
                }
            }
            context.fill();
        }
    }
}
