extern crate byteorder;
extern crate getopts;
extern crate regex;

mod stream;
mod image;
mod decoder;
mod utils;

use std::path::Path;
use std::iter::Iterator;

use utils::imageops::{
    to_diff_pair
};

use utils::file::{
    walk_test_dir,
    extract_x_y_from_name
};

use utils::cmdline_options::{
    IDPToolOptions
};

fn main() {
    let idp_tool_options = IDPToolOptions::make_new();
    if "" == &idp_tool_options.test_directory {
        return;
    } 
    idp_tool_options.print();
    
    let input_dir = Path::new( &idp_tool_options.test_directory );
    let mut file_sets = Vec::with_capacity(10);
    let _walk_result = walk_test_dir( input_dir, &mut | entries | file_sets.push( entries ) );
    let _ = match _walk_result {
      Ok( _ ) => {},
      Err( e ) => { println!("The error is : {:?}" , e  ); return; },
    };

    println!(" test_no, case x, case y,  #open_bad_pixels, open_threshold, \
               #open_bad_cols, #open_bad_rows, #short_bad_pixels, short_threshold, #total_pixels, #deadband_pix" 
            );
    for ( i, file_set ) in file_sets.iter().enumerate() {
        let diren = &file_set[0];
        let path = diren.path();
        let (x,y) = extract_x_y_from_name( &path );
        let (
           ( _open_diff_pixels_opt , _short_diff_pixels_opt ),
            ( bad_opens, number_of_bad_columns, number_of_bad_rows, bad_shorts, threshold_for_shorts , num_total, num_dead_band )
        ) = to_diff_pair( file_set, idp_tool_options.open_threshold );
        println!("{:?}, {:?}, {:?}, {:?}, {:?}, \
                  {:?}, {:?}, {:?}, {:?}, {:?}, {:?} ", 
                   i,     x,    y,   bad_opens, &idp_tool_options.open_threshold, 
                   number_of_bad_columns, number_of_bad_rows, bad_shorts, threshold_for_shorts, num_total, num_dead_band  );

    }
    println!( " \n\n DONE " );
}  
