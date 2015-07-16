extern crate byteorder;
use std::io;
use std::io::{BufReader, BufWriter};
use std::fs::{File,DirEntry};
use std::path::Path;
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
    println!("This shold create a test IDP file!");
    // let inpfile = r#"dsr_test_f32.idp"#;
    let f = match File::create( input_path ) {
        Ok( file ) => file,
        Err( msg ) => { println!("{}", msg); panic!( "room" ); }
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
        return None; // aslo check for i being in bounds
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
fn absolute_difference_of_IDP_Imges( lhs: &Path, rhs: &Path ) -> Option<Vec<Pixel> >{
    let lhs_pixels = read_test_idp( lhs ).unwrap();
    let rhs_pixels = read_test_idp( rhs ).unwrap();
    // make sure they are of the same dimensions etc..
    let pairs = lhs_pixels.iter().zip( rhs_pixels.iter());
    let diffs:Vec<Pixel> = pairs.map( | (l , r) | {
        match l.valid {
            BadType::DeadBand => Pixel{ value : 999f32, valid: BadType::Unknown  },
            _ => Pixel{ value : { let x = l.value - r.value; x.abs() }, valid: BadType::Unknown  }
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




fn vd_action(  this_entry: DirEntry, files: &mut Vec<DirEntry> ) {
    
    if fs::metadata( &this_entry.path() ).unwrap().is_dir() {
        println!( " is a dir: ignoring it {:?}\\",this_entry.path()  );
    } else {
        let this_entry_path = &this_entry.path();
        let is_reset_file = this_entry_path.to_str().unwrap().contains("PNReset");
        if is_reset_file {
            files.push( this_entry );
        }
    }
}

fn process_tail_dirs<F>(dir: &Path, cb: &mut F) -> io::Result<()> where F: FnMut(DirEntry) {
    if fs::metadata( dir ).unwrap().is_dir() {
        println!("Selecting Reset out files from {} \n", dir.display());

        for entry in try!(fs::read_dir(dir)) {
            let this_entry = try!(entry);
            let this_entry_path = &this_entry.path();
            if fs::metadata( this_entry_path ).unwrap().is_dir() {
                // cb(this_entry);
//                try!(visit_dirs(this_entry_path, cb));
            } else {
                cb(this_entry);
            }
        }
    }
    Ok(())
}


// I need a functin that can return a sequence of paths to all thee laves,
// copied from https://doc.rust-lang.org/std/fs/fn.read_dir.html and modified
// <FerrousOxide> so in cb : &mut F where F: FnMut(DirEntry) cb is a reference to a mutable ( closure that can change the args it captures, and takes a DirEntry as the arg ) ?
// <mbrubeck> FerrousOxide: Yes, though to get picky about terminology, the captured variables are not "arguments" -- they are sometimes called "upvars" (since they come from a scope "above" the closure's body)

fn visit_dirs<F>(dir: &Path, cb: &mut F) -> io::Result<()> where F: FnMut(DirEntry) {
    if fs::metadata( dir ).unwrap().is_dir() {
        println!("\n Testing Reset out files from sub dirs of : {} \n", dir.display());

        for entry in try!(fs::read_dir(dir)) {
            let this_entry = try!(entry);
            if fs::metadata( &this_entry.path() ).unwrap().is_dir() {
                let this_entry_path = &this_entry.path();
                // cb(this_entry);
                try!(process_tail_dirs(this_entry_path, cb));
            } else {
                // cb(this_entry);
            }
        }
    }
    Ok(())
}


fn main() {
    let inpfile = Path::new( r#"dsr_test_f32.idp"# );
    make_test_idp( inpfile );
    // let read_input_file = Path::new( r#"dsr_test_f32.idp"# );
    // let pixels = read_test_idp( inpfile ).unwrap();
    let lhs = Path::new( r#"L_W_X-32768_Y-32768_D15_C1717_PNResetOut_O3_BDx3_T150707131459.IDP"#);
    let rhs = Path::new( r#"L_W_X-32768_Y-32768_D15_C2525_PNResetOut_O3_BDx3_T150707131459.IDP"#);

    let diff_pixels = absolute_difference_of_IDP_Imges( lhs, rhs ).expect( "abs diff failed ");
    let bad_opens = count_bad_opens( 0.3f32, &diff_pixels );
    print!(" number of bad opens for {:?} = {:?}", inpfile, bad_opens );
    //let input_dir = Path::new( r#"\\netapp\data\projects\TQV_S1\L1_bond\test\Bondable\150707"# );
    let input_dir = Path::new( r#"test"# );
    
    let mut files = Vec::with_capacity(100);
    visit_dirs(input_dir, &mut |entry| vd_action( entry, &mut files ) ).unwrap();
    for file in files {
//        if fs::metadata( file.path() ).unwrap().is_dir() {
//            println!("{}\\ \n", file.path().display());
//        }
        println!("{} \n", file.path().display());
    }

}
