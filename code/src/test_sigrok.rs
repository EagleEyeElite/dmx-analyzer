use regex::Regex;
use sigrok::ConfigOption::{PatternMode, SampleRate};
use sigrok::{main_loop, ConfigOption, Datafeed, DriverInstance, Session, Sigrok};
use std::fmt::format;
use std::path::Iter;
use std::process::Command;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

pub fn test_sigrok(tx: Sender<u16>) {
    loop {
        let output = Command::new("sigrok-cli")
            .arg("-d")
            .arg("fx2lafw")
            .arg("--time")
            .arg("50")
            .arg("-P")
            .arg("dmx512:dmx=D0")
            .arg("-A")
            .arg("dmx512=data:channel")
            .arg("--protocol-decoder-samplenum")
            .arg("--config")
            .arg("samplerate=24MHz")
            .output()
            .expect("ls command failed to start");
        let sigrok_cli_output = String::from_utf8(output.stdout).unwrap();

        let begin_regex =
            r"(?m)^\d*-\d** dmx512-1: \d* / 0x(\d|\w)*$\n^\d*-\d** dmx512-1: \d* / 0x(\d|\w)*$\n";

        let re = Regex::new(&format!(r"{}", begin_regex)).unwrap();
        let cap = re.captures(&*sigrok_cli_output);
        if cap.is_none() {
            println!("No data found");
            continue;
        }
        let end = cap.expect("REASON").get(0).unwrap().end();

        let channel_regex =
            r"(?m)^(?P<start>\d*)-(?P<end>\d*)* dmx512-1: Channel (?P<channelNr>\d*)$\n";
        let value_regex = r"(?m)^(?P<start2>\d*)-(?P<end2>\d*)* dmx512-1: (?P<valueDez>\d*) / (?P<valueHex>0x\d|\w*)$\n";

        let re = Regex::new(&format!(r"{}{}", channel_regex, value_regex)).unwrap();
        let metaData = re
            .captures_iter(&sigrok_cli_output[..end])
            .map(|cap| {
                let channel_nr = cap
                    .name("channelNr")
                    .unwrap()
                    .as_str()
                    .parse::<u16>()
                    .unwrap();
                let value = cap
                    .name("valueDez")
                    .unwrap()
                    .as_str()
                    .parse::<u16>()
                    .unwrap();
                let sample_delta = cap.name("end2").unwrap().as_str().parse::<u32>().unwrap()
                    - cap.name("start").unwrap().as_str().parse::<u32>().unwrap();

                return ChannelMetaData { channel_nr, value, sample_delta };
            })
            .collect::<Vec<_>>();

        for data in &metaData {
            println!("ChannelNr: {}, sample delta: {}, value: {}", data.channel_nr, data.sample_delta, data.value);
        }
        tx.send(metaData[0].value.clone()).unwrap();
       // thread::sleep(Duration::from_micros(1000));
    }

}

pub struct ChannelMetaData {
    pub channel_nr: u16,
    pub value: u16,
    pub sample_delta: u32,
}

pub fn test2() {
    // Create sigrok and session.
    let mut ctx = Sigrok::new().unwrap();
    let mut ses = Session::new(&mut ctx).unwrap();
    ses.callback_add(Box::new(on_data));

    //decode(DecodableFloat);

    if let Some(driver) = ctx.drivers().iter().find(|x| x.name() == "fx2lafw") {
        let usb = ctx.init_driver(driver).unwrap();
        usb.scan();
        for device in usb.devices() {
            // Attach device.

            // Set pattern mode on digital outputs.
            if let Some(group) = device.channel_groups().get(0) {
                device.config_set(&group, &PatternMode("pattern".to_owned()));
            }

            // Set sample rate.
            for group in device.channel_groups() {
                device.config_set(&group, &SampleRate(24_000_000));
            }
            ses.add_instance(&device);
            for hmm in device.channels() {
                if hmm.index() == 0 {
                    hmm.enable();
                } else {
                    //hmm.disable();
                }
                print!("{}, {}", hmm.name(), hmm.index());
            }
        }
    }

    ses.start();
    main_loop();
    loop {}
}

fn on_data(_: &DriverInstance, data: &Datafeed) {
    match data {
        &Datafeed::Header {
            feed_version,
            start_time,
        } => {
            println!(
                "feed version {:?} start time {:?}",
                feed_version, start_time
            );
        }
        &Datafeed::Logic { unit_size, data } => {
            println!("Received {:?} bytes of {:?}-byte units.", data, unit_size);
        }
    }
}
