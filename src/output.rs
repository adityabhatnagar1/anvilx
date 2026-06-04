use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

pub fn show_banner_once() {
    let red = "\x1b[38;2;255;0;0m";
    let reset = "\x1b[0m";

    print!("{red}");

    println!(r"
                                                     ###   ######      ######
                                              ##   #####    ######    ######
                                             ####  #####     ######  ######
                                              ##     ###      ############
                                                     ###        ########
      ########    #### #####   #####   ###  ######   ###         ######
     ##    ###     ########      ###   ###   ####    ###        ########
    ###    ###     ###  ###      ###   ###   ####    ###       ##########
    ###    ###     ###  ###      ###    ##   ####    ###      ############
    ##########     ###  ###      ###   ##    ####    ###     ######  ######
     ###   ####    ###  ####     ######      ####   #####   ######    ######
");

    print!("{reset}");
}

pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(_stage: &str) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();

        let handle = thread::spawn(move || {
            let frames = ["◜", "◠", "◝", "◞", "◡", "◟"];

            let stages = [
                "Routing prompt",
                "Analyzing models",
                "Executing nodes",
                "Merging output",
            ];

            let blue  = "\x1b[38;2;52;101;164m";
            let gray  = "\x1b[38;2;211;215;207m";
            let reset = "\x1b[0m";

            print!("\x1b[?25l");
            io::stdout().flush().ok();

            let mut i = 0usize;
            let start = std::time::Instant::now();

            while !stop_thread.load(Ordering::Relaxed) {
                let spin = frames[i % frames.len()];

                let r = (((i as f32 * 0.15).sin() + 1.0) * 127.5) as u8;
                let g = ((((i as f32 * 0.15) + 2.094).sin() + 1.0) * 127.5) as u8;
                let b = ((((i as f32 * 0.15) + 4.188).sin() + 1.0) * 127.5) as u8;

                let spin_color =
                    format!("\x1b[38;2;{};{};{}m", r, g, b);

                let stage =
                    stages[(i / 20).min(stages.len() - 1)];

                let elapsed = start.elapsed().as_secs_f32();
                //let percent = ((elapsed * 12.0) as usize).min(95); removed due to occured fallacy


                print!("\r\x1b[2K");

                print!(
                    "⚒️ {}{}{} {}{}{} {}{:.1}s{}",
                    blue,
                    spin_color,
                    spin,
                    reset,
                    gray,
                    stage,
                    gray,
                    elapsed,
                    reset
                );

                io::stdout().flush().ok();

                thread::sleep(Duration::from_millis(15));

                i += 1;
            }

            let green = "\x1b[38;2;78;154;6m";

            print!("\r\x1b[2K");

            println!(
                "⚒️ {}✓{}  {}Done{}  {}{:.1}s{}",
                green,
                reset,
                green,
                reset,
                gray,
                start.elapsed().as_secs_f32(),
                reset
            );

            print!("\x1b[?25h");

            io::stdout().flush().ok();
        });

                Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}


pub fn print_response(text: &str) {
    println!("\n{}\n", text);
}