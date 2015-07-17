extern crate byteorder;
use std::io;
use std::io::{BufReader, BufWriter};
use std::fs::{File,DirEntry};
use std::path::{Path,PathBuf};
use std::option::Option;
mod stream;
mod image;
mod decoder;

use std::fs;

use stream::{
    ByteOrder,
    EndianWriter,
    SmartWriter,
    SmartReader
};

use image::error::{
    ImageResult
};

use image::other::{
    DecodingResult,
    BadType,
    Pixel
};


use decoder::{
    IDPDecoder,
    ImageDecoder
};

fn make_test_idp( input_path: &Path) {
    println!("This should create a test IDP file!");
    let f = match File::create( input_path ) {
        Ok( file ) => file,
        Err( msg ) => { println!("{}", msg); panic!( format!("Panic! unable to create file at : {:?}", input_path ) ); }
    };

    let w = BufWriter::new( &f );
    let mut wtr = SmartWriter::wrap(w, ByteOrder::LittleEndian);

    let fmt1 = 0u32;
    let fmt2 = 2u32;
      
    let number_of_columns :u32 = 1864u32;
    let number_of_rows: u32    = 1632u32;
    
    wtr.write_u32( fmt1 ).unwrap();
    wtr.write_u32( fmt2 ).unwrap();
    wtr.write_u32( number_of_columns ).unwrap();
    wtr.write_u32( number_of_rows    ).unwrap();
    for row in (0..number_of_rows ) {
        for col in ( 0..number_of_columns ) {
            wtr.write_f32( ( col * 100 + row ) as f32  ).unwrap()
        }
    }
}

fn is_dead_band( i: usize, width: usize, height: usize ) -> Option<bool> {
    if ! ( 1864 == width && 1632 == height ) {
        return None; // also check for i being in bounds
    } else {
        let row = i / width;
        let col = i % width;
        let crow = 1632 - row;
        if col >= crow && col <= ( crow + 231 ) {
            Some ( true )
        } else {
            Some ( false )
        }
    }
}

fn make_pixel_u16( (i, val ) : ( usize, &u16 ) ) -> Pixel {
    if is_dead_band( i, 1864, 1632 ).unwrap() {
        Pixel { value : val.clone() as f32, valid : BadType::DeadBand }
    }  else
    {
        Pixel { value : val.clone() as f32, valid : BadType::Unknown }
    }
}


fn make_pixel_f32( (i, val ) : ( usize, &f32 ) ) -> Pixel {
    if is_dead_band( i, 1864, 1632 ).expect( "is dead band paniced" ) {
        Pixel { value : val.clone() as f32, valid : BadType::DeadBand }
    }  else
    {
        Pixel { value : val.clone() as f32, valid : BadType::Unknown }
    }
}


fn read_test_idp( input_path: &Path) -> ImageResult<Vec<Pixel> > {
    let f = match File::open( input_path ) {
        Ok( file ) => file,
        Err( msg ) => { println!("{}", msg); panic!( "could not open input file" ); }
    };

    let bufr = BufReader::new( &f );
    let rdr = SmartReader::wrap( bufr, ByteOrder::LittleEndian );
    let mut idp_decoder = IDPDecoder::new( rdr ).unwrap();
    let decoding_result = idp_decoder.read_image().unwrap();

    let pixels: Vec<Pixel> = match decoding_result {
       DecodingResult::U16( ref _buffer) =>
       {
           let vs: &Vec<u16> = _buffer;// Make a U16 Image buffer
           let ps: Vec<Pixel> = vs.iter().enumerate().map( | (i, val) | make_pixel_u16( ( i, val ) ) ).collect();
           ps
       },
       DecodingResult::F32( ref _buffer) =>
       {
           let vs: &Vec<f32> = _buffer;// make a F32 Image buffer
           
           let ps: Vec<Pixel> = vs.iter().enumerate().map( | (i,val) | make_pixel_f32( ( i, val ) ) ).collect();
           ps
       },
   }; 

    Ok( pixels )
}
#[allow(non_snake_case)]
fn absolute_difference_of_IDP_Imges( lhs: &PathBuf, rhs: &PathBuf ) -> Option<Vec<Pixel> >{
    let lhs_pixels = read_test_idp( lhs ).unwrap();
    let rhs_pixels = read_test_idp( rhs ).unwrap();
    // make sure they are of the same dimensions etc..
    let pairs = lhs_pixels.iter().zip( rhs_pixels.iter());
    let diffs:Vec<Pixel> = pairs.map( | (left , right ) | {
        match left.valid {
            BadType::DeadBand => Pixel{ value : 999f32, valid: BadType::Unknown  },
            _ => Pixel{ value : { let x = left.value - right.value; x.abs() }, valid: BadType::Unknown  }
        }
    }).collect();
    Some( diffs )
}

fn count_bad_opens( threshold: f32,  ps: &Vec<Pixel> ) -> u64 {
    let mut count = 0u64;
    for p in ps.iter() {
        if p.valid == BadType::Unknown && threshold > p.value {
            count += 1;
        }
    }
    count
}

// ENH: instead of a vector of four,
// make two two-tuples to calculate their absolute difference
fn vd_action(  des : Vec<DirEntry>, files: &mut Vec< Vec<DirEntry> > ) {
    files.push( des );
}


// reduced to exactly two levels of dir. 
// user specifies top level, each child dir of this has all the .idp files 
// we pick only the four with PNReset in their name.
// 

fn process_tail_dirs<F>(dir: &Path, cb: &mut F) -> io::Result<()> where F: FnMut(Vec<DirEntry>) {
    if fs::metadata( dir ).unwrap().is_dir() {
        println!("Selecting Reset out files from {} \n", dir.display());

        let entries = try!(fs::read_dir(dir)); // Result<ReadDir>
        // from https://doc.rust-lang.org/std/fs/struct.ReadDir.html
        // impl Iterator for ReadDir
        // type Item = Result<DirEntry>
        // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map

        // fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F> 
        // where F: FnMut(Self::Item) -> Option<B>

        let reset_entries = entries.filter_map( | entry | { // the type of entry is Result<DirEntry>
                 let this_entry = entry.unwrap();           // unwrapping it will get us the wrapped DirEntry. But on Err it will panic
                 // try!(entry); // This will unwrap from Result but will exit the lambda on Err
                 let this_entry_path = &this_entry.path();
                 if !fs::metadata( this_entry_path ).unwrap().is_dir() && this_entry_path.to_str().unwrap().contains("PNReset") {
                     Some (this_entry )
                 } else {
                         None
                 }
             });
        let process_these = reset_entries.collect::< Vec< DirEntry> >();
        cb( process_these );
    }
    Ok(())
}


// this function enables recursively walking down/ visiting a directory tree.
// But we only need to go two levels deep.
// based on the example at https://doc.rust-lang.org/std/fs/fn.read_dir.html
// <FerrousOxide> so in cb : &mut F where F: FnMut(DirEntry) cb is a reference to a mutable ( closure that can change the args it captures, and takes a DirEntry as the arg ) ?
// <mbrubeck> FerrousOxide: Yes, though to get picky about terminology, the captured variables are not "arguments" 
// -- they are sometimes called "upvars" (since they come from a scope "above" the closure's body)

fn walk_test_dir<F>(dir: &Path, cb: &mut F) -> io::Result<()> where F: FnMut(Vec<DirEntry>) {
    if fs::metadata( dir ).unwrap().is_dir() {
        println!("\n Testing Reset out files from sub dirs of : {} \n", dir.display());

        for entry in try!(fs::read_dir(dir)) {
            let this_entry = try!(entry);
            let this_entry_path = &this_entry.path();
            if fs::metadata( this_entry_path ).unwrap().is_dir() {
                try!(process_tail_dirs(this_entry_path, cb));
            } 
        }
    }
    Ok(())
}

fn to_diff_pair( file_set : Vec<DirEntry> ) -> ( Option<Vec<Pixel> >, Option<Vec<Pixel> > ) {

    let open_test_files = &file_set.iter().filter_map( | this_entry | {
        let this_entry_path = this_entry.path();
         if this_entry_path.to_str().unwrap().contains("C1717") 
         || this_entry_path.to_str().unwrap().contains("C2525") {
             Some (this_entry )
         } else {
                 None
         }
    } ).collect::<Vec<&DirEntry>>();
    
    let open_diff_pixels =
    { 
        let lhs = open_test_files.iter().next().unwrap().path();
        let rhs = open_test_files.iter().next().unwrap().path(); 
        let open_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "abs diff failed ");
        open_diff_pix
    };
    let short_test_files = &file_set.iter().filter_map( | this_entry | {
        let this_entry_path = this_entry.path();
         if this_entry_path.to_str().unwrap().contains("C1725") 
         || this_entry_path.to_str().unwrap().contains("C2517") {
             Some (this_entry )
         } else {
                 None
         }
    } ).collect::<Vec<&DirEntry>>();
    
    let short_diff_pixels =
    { 
        let lhs = short_test_files.iter().next().unwrap().path();
        let rhs = short_test_files.iter().next().unwrap().path(); 
        let short_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "abs diff failed ");
        short_diff_pix
    };
    
    ( open_diff_pixels, short_diff_pixels );
}

// ENH get tge test_dir and threshold as cmdline args

fn main() {
    let inpfile = Path::new( r#"dsr_test_f32.idp"# );
    make_test_idp( inpfile );
    // let read_input_file = Path::new( r#"dsr_test_f32.idp"# );
    // let pixels = read_test_idp( inpfile ).unwrap();
    let lhs = PathBuf::from( r#"L_W_X-32768_Y-32768_D15_C1717_PNResetOut_O3_BDx3_T150707131459.IDP"#);
    let rhs = PathBuf::from( r#"L_W_X-32768_Y-32768_D15_C2525_PNResetOut_O3_BDx3_T150707131459.IDP"#);

    let diff_pixels = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "abs diff failed ");
    let bad_opens = count_bad_opens( 0.3f32, &diff_pixels );
    print!(" number of bad opens for {:?} = {:?}", inpfile, bad_opens );
    //let input_dir = Path::new( r#"\\netapp\data\projects\TQV_S1\L1_bond\test\Bondable\150707"# );
    let input_dir = Path::new( r#"test"# );
    
    let mut file_sets = Vec::with_capacity(10);
    walk_test_dir(input_dir, &mut | entries | vd_action( entries, &mut file_sets ) ).unwrap();
    for file_set in file_sets {
        println!( " A new set starts : " );
        for file in &file_set { 
            println!("{} \n", file.path().display());
        }
    }

}
