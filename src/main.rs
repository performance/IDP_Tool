extern crate byteorder;
extern crate simple_stats;

mod stream;
mod image;
mod decoder;
mod utils;

use std::path::Path;

use utils::imageops::{
    to_diff_pair
};

use utils::file::{
    walk_test_dir,
    vd_action
};


// ENH get tge test_dir and threshold as cmdline args
fn main() {
    let input_dir = Path::new( r#"test"# );
    let mut file_sets = Vec::with_capacity(10);
    walk_test_dir(input_dir, &mut | entries | vd_action( entries, &mut file_sets ) ).unwrap();
    for file_set in file_sets {
        println!( " A new set starts : " );
        to_diff_pair( file_set );
    }
    println!( " \n\n DONE " );
} 
