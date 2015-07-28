use getopts;
use getopts::Options;
use getopts::Matches;
use std::env;

pub struct IDPToolOptions {
    pub test_directory  : String,
    pub open_threshold  : f32,
    pub short_threshold : f32,
    pub ignore_edges    : usize,
}


impl IDPToolOptions {
    fn new() -> IDPToolOptions {
        IDPToolOptions {
            test_directory  : "".to_string(),
            open_threshold  : 0f32,
            short_threshold : 0f32,
            ignore_edges    : 0usize,
        }
    }

    fn mnew( matches : &getopts::Matches ) -> IDPToolOptions {
        IDPToolOptions {
            test_directory  : matches.opt_str( "t" ).unwrap_or( "test".to_string() ), 
            open_threshold  : matches.opt_str( "o" ).unwrap_or( "0.3".to_string()  ).trim().parse::<f32  >().ok().unwrap_or( 0.0f32 ), 
            short_threshold : matches.opt_str( "s" ).unwrap_or( "0.75".to_string() ).trim().parse::<f32  >().ok().unwrap_or( 0.0f32 ), 
            ignore_edges    : matches.opt_str( "i" ).unwrap_or( "0".to_string()    ).trim().parse::<usize>().ok().unwrap_or( 0usize ), 
            
        }
    }   
    pub fn make_new() -> IDPToolOptions {
       let args: Vec<String> = env::args().collect();
       match cmdline_options( &args ) {
           Some( matches ) => IDPToolOptions::mnew( &matches ),
           None            => IDPToolOptions::new( ),
       }
   }

    pub fn print( &self) {
        println!("The following test options will be used: " );
        println!("test_directory  : {:?}", self.test_directory  );
        println!("open_threshold  : {:?}", self.open_threshold  );
        println!("short_threshold : {:?}", self.short_threshold );
        println!("ignore_edges    : {:?}", self.ignore_edges    );
    }
}

fn get_current_version() -> String {
    let key = "CARGO_PKG_VERSION";
    match env::var(key) {
        Ok(val) => val,
        Err(e)  => "unkwon",
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} version: {} [ -t <test_dir> ] [ -o <open_threshold> ] [ -s <short_threshold> ] [ -i <ignore_edges> ]", program, get_current_version() );
    print!("{}", opts.usage(&brief));
}


fn cmdline_options( _args : &Vec<String> ) -> Option< Matches > {
    let mut opts = Options::new();
    opts.optopt("t", "test_dir",        "Test area with each sub dir containing idp images", "test" );
    opts.optopt("o", "open_threshold",  "Threshold to use for open test.",                   "0.3"  );
    opts.optopt("s", "short_threshold", "% of median to use as threshold for short test.",   "0.75" );
    opts.optopt("i", "ignore_edges",    "number of rows/cols to ignore along the edges.",    "0"    );
    opts.optflag("v", "version",        "print the current verison"                                 );
    opts.optflag("h", "help",           "print this help menu"                                      );

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err( _f) => { 
            print_usage(&program, &opts); 
            panic!( format!("Please check that the arguments are correct : {:?} ", &args ) );
        }
    };

    if matches.opt_present("v") {
        println!(" {} version: {} ", &program, get_current_version() );
    }

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return None;
    }
    return Some ( matches );
}

