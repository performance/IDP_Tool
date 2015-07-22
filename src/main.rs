extern crate byteorder;

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


// ENH get tge test_dir and threshold as cmdline args
fn main() {
    let input_dir = Path::new( r#"test"# );
    let mut file_sets = Vec::with_capacity(10);
    walk_test_dir(input_dir, &mut | entries | file_sets.push( entries ) ).unwrap();

    for ( i, file_set ) in file_sets.iter().enumerate() {
        println!( "--------------------- ------------------------- ------------------------------ ");
        println!( "\n\n Stats for set number : {:?}", i );
        to_diff_pair( file_set );
    }
    println!( " \n\n DONE " );
} 
