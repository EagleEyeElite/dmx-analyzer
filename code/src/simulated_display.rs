#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod render_engine;
mod test_sigrok;

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use cascade::cascade;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder,
    SimulatorDisplay,
    SimulatorEvent,
    Window,
    sdl2::Keycode};
use render_engine::{Engine, Parameter};
use std::time::{Duration, Instant};
use crate::test_sigrok::test_sigrok;

use std::ffi::CString;
use std::os::raw::{c_char, c_int};


#[repr(transparent)]
struct RustObject {
    a: i32,
    // Other members...
}

extern "C" fn callback(target: *mut RustObject, a: i32) {
    println!("I'm called from C with value {0}", a);
    unsafe {
        // Update the value in RustObject with the value received from the callback:
        (*target).a = a;
    }
}

#[link(name = "saleaeLogic", kind = "static")]
extern {
    fn mainC(target: *mut RustObject,
             cb: extern fn(*mut RustObject, i32)) -> i32;
}

fn main() {
    // Create the object that will be referenced in the callback:
    let mut rust_object = Box::new(RustObject { a: 5 });

    unsafe {
        mainC(&mut *rust_object, callback);
    }
    println!("{}", rust_object.a);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        test_sigrok(tx);
    });

    let output_settings = OutputSettingsBuilder::new()
        .pixel_spacing(0)
        .scale(2)
        .build();
    let mut display = SimulatorDisplay::new(
        Size::new(300, 300)
    );
    let mut window = cascade! {
        Window::new("Led Matrix", &output_settings);
        ..update(&display);
    };

    let str = [
        "src/objects/cube.obj",
        "src/objects/video_ship.obj",
        "src/objects/teapot.obj"
    ];
    let mut engine = Engine::new(&str[0], &mut display);
    let mut last: Instant = Instant::now();

    let mut rotation_now = 0.0;
    'running: loop {
        let now = Instant::now();
        let mut parameter = Parameter{
            eye: Default::default(),
            rotation: rotation_now,
            elapsed_time: now - last,
            print_state: false
        };
        for event in window.events() {
            match event {
                SimulatorEvent::Quit =>
                    break 'running,
                SimulatorEvent::KeyDown { keycode, .. } =>
                    match keycode {
                        Keycode::Right => parameter.eye.x = -1.0,
                        Keycode::Left => parameter.eye.x = 1.0,
                        Keycode::Down => parameter.eye.z = -1.0,
                        Keycode::Up => parameter.eye.z = 1.0,
                        Keycode::W => parameter.eye.y = -0.1,
                        Keycode::S => parameter.eye.y = 0.1,
                        Keycode::N => parameter.rotation = 0.1,
                        Keycode::M => parameter.rotation = -0.1,
                        Keycode::P => parameter.print_state = true,
                        _ => {}
                    },
                _ => {}
            }
        }

        let received = rx.try_recv();
        if received.is_ok() {
            let received = received.unwrap();
            rotation_now = received as f32 / 255.0;
        }

        engine.on_user_update(&mut display, parameter);
        last = now;
        window.update(&display);
        display.clear(Rgb888::new(0, 0, 0)).unwrap();
    }
    println!("{}", rust_object.a);
}
