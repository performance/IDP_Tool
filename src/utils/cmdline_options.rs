use getopts;
use getopts::Options;
use getopts::Matches;
use std::env;

pub struct IDPToolOptions {
    pub test_directory : String,
    pub open_threshold : f32,
}


impl IDPToolOptions {
   // fn new() -> IDPToolOptions {
   //     IDPToolOptions {
   //         test_directory  : "".to_string(),
   //         open_threshold    : 0f32,
   //     }
   // }

    fn mnew( matches : &getopts::Matches ) -> IDPToolOptions {
        IDPToolOptions {
            test_directory : matches.opt_str( "t" ).unwrap_or( "test".to_string() ), 
            open_threshold : matches.opt_str( "o" ).unwrap_or( "0.3".to_string() ).trim().parse::<f32>().ok().unwrap_or( 0.0f32 ), 
        }
    }   
    pub fn make_new() -> IDPToolOptions {
       let args: Vec<String> = env::args().collect();
       let matches = cmdline_options( &args ).expect( "" );
       IDPToolOptions::mnew( &matches )
   }

    pub fn print( &self) {
        println!("The following image options will be used: " );
        println!("test_directory : {:?}", self.test_directory );
        println!("open_threshold : {:?}", self.open_threshold );
    }
}


fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} <columns> <rows> <-f|-s|-e>", program);
    print!("{}", opts.usage(&brief));
}

fn cmdline_options( _args : &Vec<String> ) -> Option< Matches > {
    let mut opts = Options::new();
    opts.optopt("t", "test_dir",       "Test area with each sub dir containing idp images", "test" );
    opts.optopt("o", "open_threshold", "Threshold to use for open test.",                   "0.3"  );
    opts.optflag("h", "help",          "print this help menu"                                      );

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { print_usage(&program, &opts); panic!(f.to_string());  }
    };
    
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return None;
    }
    return Some ( matches );
}

