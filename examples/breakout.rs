//! A simple example that demonstrates capturing window and input events.
use std::collections::HashSet;

use coffee::graphics::{
    Color, Font, Frame, Image, Point, Quad, Rectangle, Text, Vector, Window,
    WindowSettings,
};
use coffee::input;
use coffee::load::{loading_screen, Join, LoadingScreen, Task};
use coffee::{Game, Result, Timer};

fn main() -> Result<()> {
    BreakoutExample::run(WindowSettings {
        title: String::from("Breakout - Coffee"),
        size: (800, 600),
        resizable: false,
        fullscreen: false,
    })
}

struct Input {
    cursor_position: Point,
    mouse_wheel: Point,
    keys_pressed: HashSet<input::KeyCode>,
    mouse_buttons_pressed: HashSet<input::MouseButton>,
    text_buffer: String,
}

impl Input {
    fn new() -> Input {
        Input {
            cursor_position: Point::new(0.0, 0.0),
            mouse_wheel: Point::new(0.0, 0.0),
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            text_buffer: String::new(),
        }
    }
}

struct View {
    palette: Image,
    font: Font,
    ball: Image,
    paddle: Image,
}

impl View {
    const COLORS: [Color; 1] = [Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }];

    fn load() -> Task<View> {
        (
            Task::using_gpu(|gpu| Image::from_colors(gpu, &Self::COLORS)),
            Font::load(include_bytes!(
                "../resources/font/Retro-Gaming.ttf"
            )),
            Image::load("resources/ball.png"),
            Image::load("resources/paddle.png")
        )
            .join()
            .map(|(palette, font, ball, paddle)| View { palette, font, ball, paddle })
    }
}

struct BreakoutExample {
    cursor_position: Point,
    keys_pressed: HashSet<input::KeyCode>,
    mouse_buttons_pressed: HashSet<input::MouseButton>,
    ball: Ball,
    paddle: Paddle,
    bricks: Vec<Brick>,
    bounds: Rectangle<f32>,
    score: u32,
}

struct Ball {
    radius: f32,
    position: Point,
    speed: f32,
    normal: Vector,
}

#[derive(Copy, Clone)]
struct Brick {
    position: Point,
    size: (f32, f32),
}

struct Paddle {
    size: (f32, f32),
    position: f32,
}

impl BreakoutExample {
    const MAX_TEXTSIZE: usize = 40;

    fn incidence(incoming: Vector, normal: Vector) -> Vector {

        //use coffee::nalgebra::normalize;

        let dot = (incoming.x * normal.x) + (incoming.y * normal.y);

        println!("dot {:?}", dot);

        Vector::new(dot.cos(), dot.sin())

    }

    /// Circle/Rectangle collision detection
    /// Source: https://stackoverflow.com/a/402010
    fn intersects(ball: &Ball, rect: Rectangle<f32>) -> bool {
        
        let mut circle_distance = Point::new(0.0, 0.0);
        
        circle_distance.x = f32::abs(ball.position.x - rect.x);
        circle_distance.y = f32::abs(ball.position.y - rect.y);

        if (circle_distance.x > (rect.width/2.0 + ball.radius)) { return false; }
        if (circle_distance.y > (rect.height/2.0 + ball.radius)) { return false; }

        if (circle_distance.x <= (rect.width/2.0)) { return true; } 
        if (circle_distance.y <= (rect.height/2.0)) { return true; }

        let corner_distance_sq = (circle_distance.x - rect.width/2.0).powf(2.0) +
                            (circle_distance.y - rect.height/2.0).powf(2.0);

        corner_distance_sq <= (ball.radius.powf(2.0))
    }
}

impl Game for BreakoutExample {
    type View = View;
    type Input = Input;

    const TICKS_PER_SECOND: u16 = 60;

    fn new(
        window: &mut Window,
    ) -> Result<(BreakoutExample, Self::View, Self::Input)> {
        let task = Task::stage("Loading font...", View::load());

        let mut loading_screen = loading_screen::ProgressBar::new(window.gpu());
        let view = loading_screen.run(task, window)?;

        let brick_size = (75.0, 30.0);

        let bricks = {
            let mut bricks = Vec::new();

            for x in 0..10 {
                for y in 0..5 {
                    let x = x as f32;
                    let y = y as f32;

                    bricks.push(Brick {
                        position: Point::new(x * brick_size.0, y * brick_size.1),
                        size: brick_size,
                    })
                }
            }

            bricks
        };

        Ok((
            BreakoutExample {
                cursor_position: Point::new(0.0, 0.0),
                keys_pressed: HashSet::new(),
                mouse_buttons_pressed: HashSet::new(),
                paddle: Paddle {
                    size: (64.0 * 2.0, 12.0 * 2.0),
                    position: 0.0,
                },
                ball: Ball {
                    position: Point::new(200.0, 200.0),
                    radius: 8.0,
                    speed: 10.0,
                    normal: Vector::new(0.8, 0.2)
                },
                bounds: Rectangle {
                    x: 0.0,
                    y: 32.0,
                    width: window.width(),
                    height: window.height(),
                },
                score: 0,
                bricks,
            },
            view,
            Input::new(),
        ))
    }

    fn on_input(&self, input: &mut Input, event: input::Event) {
        match event {
            input::Event::CursorMoved { x, y } => {
                input.cursor_position = Point::new(x, y);
            }
            input::Event::TextInput { character } => {
                input.text_buffer.push(character);
            }
            input::Event::KeyboardInput { key_code, state } => match state {
                input::ButtonState::Pressed => {
                    input.keys_pressed.insert(key_code);
                }
                input::ButtonState::Released => {
                    input.keys_pressed.remove(&key_code);
                }
            },
            input::Event::MouseInput { state, button } => match state {
                input::ButtonState::Pressed => {
                    input.mouse_buttons_pressed.insert(button);
                }
                input::ButtonState::Released => {
                    input.mouse_buttons_pressed.remove(&button);
                }
            },
            _ => {}
        }
    }

    fn update(&mut self, _view: &Self::View, window: &Window) {

        self.ball.position += self.ball.speed * self.ball.normal; 

        // Wall bounces
        if self.ball.position.x + self.ball.radius >= self.bounds.width {
            self.ball.normal.x = -self.ball.normal.x;
        }
        if self.ball.position.x - self.ball.radius <= self.bounds.x {
            self.ball.normal.x = -self.ball.normal.x;
        }
        if self.ball.position.y + self.ball.radius >= self.bounds.height {
            self.ball.normal.y = -self.ball.normal.y;
        }
        if self.ball.position.y + self.ball.radius <= self.bounds.y {
            self.ball.normal.y = -self.ball.normal.y;
        }

        // Brick collisions
        /*self.bricks = self.bricks.iter().cloned().filter(|brick|{
            !BreakoutExample::intersects(&self.ball, Rectangle {
                x: brick.position.x,
                y: brick.position.y,
                width: brick.size.0,
                height: brick.size.1,
            })
        }).collect();*/
        
        for (index, brick) in self.bricks.iter().enumerate() {
            if BreakoutExample::intersects(&self.ball, Rectangle {
                x: brick.position.x,
                y: brick.position.y,
                width: brick.size.0,
                height: brick.size.1,
            }) {
                self.bricks.remove(index);
                break;
            }
        }

        

    }

    fn interact(
        &mut self,
        input: &mut Input,
        _view: &mut View,
        _window: &mut Window,
    ) {
        self.cursor_position = input.cursor_position;
        self.keys_pressed = input.keys_pressed.clone();
        self.mouse_buttons_pressed = input.mouse_buttons_pressed.clone();

        if input.keys_pressed.contains(&input::KeyCode::D) {
            self.paddle.position += 10.0;
        }
        if input.keys_pressed.contains(&input::KeyCode::A) {
            self.paddle.position -= 10.0;
        }
    }

    fn draw(&self, view: &mut Self::View, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::new(51.0 / 255.0, 153.0 / 255.0, 218.0 / 255.0, 1.0));

        // This closure simplifies some of the boilerplate.
        let mut add_aligned_text =
            |label: String, content: String, x: f32, y: f32| {
                view.font.add(Text {
                    content: label,
                    position: Point::new(x, y),
                    bounds: (frame.width(), frame.height()),
                    size: 20.0,
                    color: Color::WHITE,
                });
                view.font.add(Text {
                    content: content,
                    position: Point::new(x + 260.0, y),
                    bounds: (frame.width(), frame.height()),
                    size: 20.0,
                    color: Color::WHITE,
                });
            };

        

        for brick in &self.bricks {
            view.palette.draw(
                Quad {
                    source: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 1.0,
                        height: 1.0,
                    },
                    position: brick.position,
                    size: brick.size,
                },
                &mut frame.as_target(),
            );
        }

        // Draw a small square at the mouse cursor's position.
        view.paddle.draw(
            Quad {
                source: Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                position: Point::new(self.paddle.position, frame.height() - 40.0),
                size: self.paddle.size,
            },
            &mut frame.as_target(),
        );

        view.ball.draw(
            Quad {
                source: Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                position: self.ball.position,
                size: (self.ball.radius * 2.0, self.ball.radius * 2.0),
            },
            &mut frame.as_target(),
        );

        // Draw UI
        view.font.add(Text {
            content: format!("Score: {}", self.score).to_string(),
            position: Point::new(0.0, 0.0),
            bounds: (frame.width(), frame.height()),
            size: 14.0,
            color: Color::WHITE,
        });

        view.font.add(Text {
            content: String::from("Breakout"),
            position: Point::new(frame.width() - 150.0, 0.0),
            bounds: (frame.width(), frame.height()),
            size: 14.0,
            color: Color::WHITE,
        });

        view.font.draw(&mut frame.as_target());

    }
}
