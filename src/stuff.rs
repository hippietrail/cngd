use std::env;

#[macro_export]
macro_rules! vprintln {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! evprintln {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!($($arg)*);
        }
    };
}

pub struct Cfg {
    pub verbose: bool,
    pub auto_cluster: bool,
    pub user_specified_cores: Vec<String>,
}

pub fn cmdline() -> Cfg {
    let mut verbose = false; // Enable verbose/debug output
    let mut auto_cluster = true; // Use automatic clustering for core selection
    let mut user_specified_cores: Vec<String> = Vec::new(); // Cores explicitly provided by user

    for arg in env::args().skip(1) {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else if arg == "--no-auto-cluster" {
            auto_cluster = false;
        } else {
            // Any unrecognized argument is treated as a user-specified core word or phrase
            evprintln!(verbose, "🤞: {}", arg);
            user_specified_cores.push(arg);
        }
    }

    // (verbose, auto_cluster, user_specified_cores)
    Cfg {
        verbose,
        auto_cluster,
        user_specified_cores,
    }
}

pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn hsl_to_colour(h: f32, s: f32, l: f32) -> Colour {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Colour {
        r: ((r1 + m) * 255.0).round() as u8,
        g: ((g1 + m) * 255.0).round() as u8,
        b: ((b1 + m) * 255.0).round() as u8,
    }
}
