use coffee::graphics::{Color, Frame, Window, WindowSettings, Image, Rectangle, Quad, Point};
use coffee::load::{Task, Join};
use coffee::{Game, Result, Timer};
use coffee::Debug;

use nalgebra as na;
use na::{Point2, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{ColliderDesc, RigidBodyDesc};
use nphysics2d::world::World;

fn main() -> Result<()> {
    Example::run(WindowSettings {
        title: String::from("Physics - Coffee"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
    })
}

struct Example {
    palette: Image,
    world: World<f32>,
}

impl Example {
    const PRUSSIAN_BLUE: Color = Color {
        r: 0.0,
        g: 0.1922,
        b: 0.3255,
        a: 1.0,
    };
}

impl Game for Example {

    type Input = (); // No input data
    type LoadingScreen = (); // No loading screen

    fn load(_window: &Window) -> Task<Example> {
        // Load your game assets here. Check out the `load` module!
        (
            Task::using_gpu(|gpu| {
                Image::from_colors(gpu, &[Self::PRUSSIAN_BLUE])
            }),
            Task::new(|| {

                /*
                * World
                */
                let mut world = World::new();
                world.set_gravity(Vector2::new(0.0, 9.81));

                /*
                * Ground
                */
                let ground_size = 200.0;
                let ground_shape =
                    ShapeHandle::new(Cuboid::new(Vector2::new(ground_size, 1.0)));

                ColliderDesc::new(ground_shape)
                    .translation(Vector2::y() * 200.0)
                    .build(&mut world);

                /*
                * Create the boxes
                */
                let num = 10;
                let rad = 10.0;

                let cuboid = ShapeHandle::new(Cuboid::new(Vector2::repeat(rad)));
                let collider_desc = ColliderDesc::new(cuboid)
                    .density(1.0);

                let mut rb_desc = RigidBodyDesc::new()
                    .collider(&collider_desc);

                let shift = (rad + collider_desc.get_margin()) * 2.0;
                let centerx = shift * (num as f32) / 2.0;
                let centery = shift / 2.0;

                for i in 0usize..num {
                    for j in 0..num {
                        let x = i as f32 * shift - centerx;
                        let y = j as f32 * shift + centery;

                        // Build the rigid body and its collider.
                        rb_desc
                            .set_translation(Vector2::new(x, y))
                            .build(&mut world);
                    }
                }

                Ok(world)
            })
        ).join()
            .map(|(palette, world)| Example { palette, world })


    }

    fn update(&mut self, _window: &Window) {

        for _ in 0..5 {
            self.world.step();
        }

    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(Color::BLACK);

        let target = &mut frame.as_target();

        for collider in self.world.colliders() {
            
            let position = collider.position();

            let position = position.translation.vector;

            let shape = collider.shape();

            // maybe use this instead of half_extents? Not sure.
            let shift = (0.1 + collider.margin()) * 2.0;
            
            if let Some(cubioid) = shape.as_shape::<Cuboid<f32>>() {

                let half_extents = cubioid.half_extents();

                self.palette.draw(
                    Quad {
                        source: Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: 1.0,
                            height: 1.0,
                        },
                        position: Point::new(position[0], position[1]),
                        size: (half_extents[0], half_extents[1]),
                    },
                    target,
                );

            }
            
        }

        // Draw your game here. Check out the `graphics` module!
    }
}