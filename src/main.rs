extern crate byteorder;
extern crate getopts;


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
};

use utils::cmdline_options::{
    IDPToolOptions
};

fn main() {
    let idp_tool_options = IDPToolOptions::make_new();
    idp_tool_options.print();


    let input_dir = Path::new( &idp_tool_options.test_directory );
    let mut file_sets = Vec::with_capacity(10);
    walk_test_dir( input_dir, &mut | entries | file_sets.push( entries ) ).unwrap();

    for ( i, file_set ) in file_sets.iter().enumerate() {
        println!( "--------------------- ------------------------- ------------------------------ ");
        println!( "\n\n Stats for set number : {:?}", i );
        to_diff_pair( file_set, idp_tool_options.open_threshold );
    }
    println!( " \n\n DONE " );
}  
