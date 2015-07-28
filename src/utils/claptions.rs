use clap::{Arg, App};
use std::env;

pub struct IDPToolOptions {
    pub test_directory  : String,
    pub open_threshold  : f32,
    pub short_threshold : f32,
    pub ignore_edges    : usize,
}


impl IDPToolOptions {
    // fn new() -> IDPToolOptions {
    //     IDPToolOptions {
    //         test_directory  : "".to_string(),
    //         open_threshold  : 0f32,
    //         short_threshold : 0f32,
    //         ignore_edges    : 0usize,
    //     }
    // }
    pub fn make_new() -> IDPToolOptions {
       from_cmdline_options()
   }

    pub fn print( &self) {
        println!("The following test options will be used: " );
        println!("test_directory  : {:?}", self.test_directory  );
        println!("open_threshold  : {:?}", self.open_threshold  );
        println!("short_threshold : {:?}", self.short_threshold );
        println!("ignore_edges    : {:?}", self.ignore_edges    );
    }
}

fn from_cmdline_options() -> IDPToolOptions {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = App::new(&program)
                          .version(&crate_version!()[..])
                          .about("Tool to measure Open and Short Bad Bump Bonds from IDP images.")
                          .arg(Arg::with_name("test_directory")
                               .short("t")
                               .long("test_dir")
                               .help("Test area with each sub dir containing idp images.")
                               .required( true )
                               .takes_value(true)
                           )
                          .arg(Arg::with_name("open_threshold")
                               .short("o")
                               .long("open_threshold")
                               .help("Threshold to use for open test.")
                               .required(false)
                               .takes_value(true)
                               )
                          .arg(Arg::with_name("short_threshold")
                               .short("s")
                               .long("short_threshold")
                               .help("% of the median to use as threshold for short test.")
                               .required(false)
                               .takes_value(true)
                               )
                          .arg(Arg::with_name("ignore_edges")
                               .short("i")
                               .long("ignore_edges")
                               .help("number of rows/cols to ignore along the edges.")
                               .required(false)
                               .takes_value(true)
                               )
                          .get_matches();
    let test_directory  = matches.value_of( "test_directory"  ).unwrap_or( "test" ).to_string();
    let open_threshold  = matches.value_of( "open_threshold"  ).unwrap_or( "0.3"  ).trim().parse::<f32  >().ok().unwrap_or( 0.0f32 ); 
    let short_threshold = matches.value_of( "short_threshold" ).unwrap_or( "0.75" ).trim().parse::<f32  >().ok().unwrap_or( 0.0f32 ); 
    let ignore_edges    = matches.value_of( "ignore_edges"    ).unwrap_or( "0"    ).trim().parse::<usize>().ok().unwrap_or( 0usize );

    IDPToolOptions {
        test_directory  : test_directory,
        open_threshold  : open_threshold,
        short_threshold : short_threshold,
        ignore_edges    : ignore_edges,
    }
}