/*
gst-launch-1.0 v4l2src device="/dev/video0" ! videoconvert ! clockoverlay ! \
    x264enc tune=zerolatency ! mpegtsmux ! \
    hlssink playlist-root=http://127.0.0.1:8080 location=/home/CHANGEME/gstreamer/segment_%05d.mp4 target-duration=5 max-files=5
*/

use colored::*;
pub use gstreamer;
use gstreamer::prelude::ElementExt;
use gstreamer::prelude::GstObjectExt;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut recording_source_input = "";
    if args.len() > 1 {
        recording_source_input = &args[1];
    }

    let mut base_args: String = "".to_string();

    match recording_source_input.as_ref() {
        "options" => {
            println!("\nSecond argument options:\n  {} for linux webcams or tv card\n  {} for capturing X Display",
            "camera".yellow(), "desktop".yellow());

            process::exit(1);
        }
        "camera" => base_args.push_str("v4l2src device=/dev/video0"),
        "desktop" | _ => base_args.push_str("ximagesrc"),
    }

    base_args.push_str(
        " ! videoconvert ! \
        x264enc tune=zerolatency ! mpegtsmux ! \
        hlssink location=recordings/recording_%03d.mp4 target-duration=5",
    );

    match fs::create_dir_all("recordings") {
        Ok(_) => (),
        Err(e) => println!("Error creating dir: {}", e.to_string()),
    }

    recorder(&base_args);
}

fn recorder(launch_options: &str) {
    println!("Started Recording");
    let main_loop = glib::MainLoop::new(None, false);

    gstreamer::init().unwrap();
    let pipeline = gstreamer::parse_launch(&launch_options).unwrap();
    let bus = pipeline.bus().unwrap();

    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let main_loop_clone = main_loop.clone();

    bus.add_watch(move |_, msg| {
        use gstreamer::MessageView;

        let main_loop = &main_loop_clone;
        match msg.view() {
            MessageView::Eos(..) => main_loop.quit(),
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                main_loop.quit();
            }
            _ => (),
        };

        glib::Continue(true)
    })
    .expect("Failed to add bus watch");

    main_loop.run();

    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set pipeline to NULL state");

    bus.remove_watch().unwrap();
}
