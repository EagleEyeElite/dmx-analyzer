use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};
use std::{fmt::Debug, thread};
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver};

use embedded_graphics::primitives::{Line, PrimitiveStyle};
use crate::logic_analyzer::{DmxPacket, get_dmx_data};

pub struct Controller {
    pub rx: Receiver<DmxPacket>,
}

#[derive(Debug, Clone, Copy)]
pub struct Parameter {
    pub channels: DmxPacket,
}

impl Controller {
    pub fn new(from_device: bool, set_u_dmx_output: bool) -> Controller {
        if set_u_dmx_output {
            // set dmx output -> ch1: 0xAD, ch2: 0xBE, ch3: 0xD0
            let mut command = Command::new("./../uDMX commandline/uDMX");
            let start_dmx = command.arg("0"); // start address, channel 1 for dmx analyzer

            let my_array = [
                150,  200,  0x00,
                0x10, 0x14, 0x18, 0x1C,
                0x20, 0x24, 0x28, 0x2C,
                0x30, 0x34, 0x38, 0x3C,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x10, 0x30, 0x10, 0x40,
                0x15, 0x20, 0x05, 0x40,
            ];
            for item in my_array.iter() {
                start_dmx.arg(item.to_string());
            }

            let output = start_dmx.output().unwrap();
            if !output.status.success() {
                println!("{}", String::from_utf8(output.stdout).unwrap());
                panic!("couldnt start uDMX, try reinserting the usb to dmx adapter")
            }
        }

        let (tx, rx) = mpsc::channel::<DmxPacket>();
        thread::spawn(move || {
            get_dmx_data(tx, from_device);
        });
        return Controller{rx};
    }

    pub fn on_user_update<D>(&mut self, display: &mut D, parameter: Parameter)
    where
        D: DrawTarget<Color = Rgb888>,
        D::Error: Debug,
    {
        let _screen_width = display.bounding_box().size.width;
        let screen_height = display.bounding_box().size.height;

        let channels = parameter.channels.channels;
        for (index, value) in channels.iter().enumerate() {
            let x_start = index as i32;
            let y_start = screen_height as i32;
            let y_end = y_start - (*value as f32 * (64.0 / 255.0)) as i32;

            Line::new(Point::new(x_start, y_start), Point::new(x_start, y_end))
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
                .draw(display).unwrap();
        }
    }
}
