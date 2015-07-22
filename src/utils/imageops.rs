extern crate simple_stats;

use std::fs::{DirEntry};

use image::other::{
    BadType,
    Pixel
};

use utils::file::{
    absolute_difference_of_IDP_Imges
};


fn mark_open_bads( threshold: f32, ps: &Vec<Pixel> )  ->  ( Option<Vec<Pixel> >, u64 ) {
    let total_pix = ps.len();
    let mut count = 0u64;
    let mut marked_pixels : Vec<Pixel> = Vec::with_capacity(total_pix);
    for p in ps.iter() {
        let mask_pix = 
        if p.valid == BadType::Unknown && threshold > p.value {
            count += 1;
            Pixel{ value: p.value, valid: BadType::OpenBad }
        } else {
            Pixel{ value: p.value, valid: p.valid }
        };
        marked_pixels.push( mask_pix );
    }
    ( Some( marked_pixels ), count )
}


fn count_bad_pixels( threshold: f32, ms: &Vec<Pixel>, ps: &Vec<Pixel> ) -> u64 {
    let mut count = 0u64;

    for ( m, p ) in ms.iter().zip( ps.iter() ) {
        if m.valid == BadType::Unknown && p.valid == BadType::Unknown && threshold > p.value {
            count += 1;
        }
    }
    count
}

// TODO: this name sux, change it to median_of_unmasked_pixel_values or something better
fn collect_unmasked_pixel_values_median ( ms: &Vec<Pixel>, ps: &Vec<Pixel> ) -> Option<f32> {
    // let mut unmasked_pixel_values : Vec<f32> = Vec::with_capacity(ps.len() ); //  this will over allocate
    // for ( m, p ) in ms.iter().zip( ps.iter() ) {
    //     if m.valid == BadType::Unknown {
    //         unmasked_pixel_values.push( p.value );
    //     }
    // }
    let threshold_for_shorts  = {
        let unmasked_pixel_values: Vec<f32> = ms.iter().zip( ps.iter() ).filter_map( 
            | ( m,p ) | if m.valid == BadType::Unknown { Some( p.value ) } else { None } 
        ).collect();
        simple_stats::median( &unmasked_pixel_values )
    };
    Some( threshold_for_shorts )
}

fn pixels_to_mask( ps: &Vec<Pixel>,  width: usize, height: usize  ) ->  Option<Vec<Pixel> >{
    let total_pix = ps.len();
    // assert that this is equal to width times height
    
    let mut bad_pix_in_row : Vec<usize> = Vec::with_capacity( height );
    let mut bad_pix_in_col : Vec<usize> = Vec::with_capacity( width );
    for _row in 0..height {
        bad_pix_in_row.push( 0 );
    }
    for _col in 0..width {
        bad_pix_in_col.push( 0 );
    }

    {
        let mut pit = ps.iter();
        for row in 0..height {
            for col in 0..width{
                if BadType::OpenBad == pit.next().unwrap().valid {
                        bad_pix_in_col[ col ] += 1;
                        bad_pix_in_row[ row ] += 1;
                }
            }
        }
    }
    // println!("half height = {:?}", ( height / 2 ) );
    // for r in &bad_pix_in_col {
    //     print!("{:?}, ", r );
    // }

    // println!("\nhalf width = {:?}", ( width / 2 ) );
    // for c in &bad_pix_in_row { 
    //     print!("{:?}, ", c );
    // }
    // println!( "at line: {:?} bad_pix_in_col = \n{:?}, \nbad_pix_in_row = \n{:?}  ", line!(), &bad_pix_in_col, &bad_pix_in_row );
    
    let bad_cols : Vec<bool> = bad_pix_in_col.iter().map( |col_count| *col_count > ( height / 2 ) ).collect();
    let bad_rows : Vec<bool> = bad_pix_in_row.iter().map( |row_count| *row_count > ( width  / 2 ) ).collect();
    
    // println!( "\nat line: {:?} bad_cols = \n{:?}, \nbad_rows = \n{:?}", line!(), &bad_cols, &bad_rows );
    
    let mut mask_pixels : Vec<Pixel> = Vec::with_capacity(total_pix);
    // println!( "at line: {:?} ", line!() );
    let mut pit = ps.iter();
    // println!( "at line: {:?} ", line!() );
    let mut num_bad_col = 0u64;
    let mut num_bad_row = 0u64;
    let mut num_bad_both = 0u64;
    let mut num_dead_band = 0u64;
    let mut num_unknown = 0u64; 
    
    for row in 0..height {
        for col in 0..width{
            // println!( "at line: {:?} ", line!() );
            let mask_pix = match ( bad_cols[ col ], bad_rows[ row ] ) {
                ( true,  true  ) => { num_bad_both += 1; Pixel{ value : 999f32, valid: BadType::OpenBadBoth } },
                ( true,  false ) => { num_bad_col  += 1; Pixel{ value : 999f32, valid: BadType::OpenBadCol  } },
                ( false, true  ) => { num_bad_row  += 1; Pixel{ value : 999f32, valid: BadType::OpenBadRow  } },
                ( false, false ) => {
                    // println!( "at line: {:?} ", line!() );
                    let this_pix = pit.next().unwrap();
                    let this_valid = this_pix.valid;
                    let this_value = this_pix.value;
                    if BadType::DeadBand == this_valid {
                        num_dead_band += 1;
                    }
                    else
                    {
                        num_unknown += 1;
                    }
                    Pixel{ value : this_value, valid: this_valid  }
                },
            };
            // println!( "at line: {:?} ", line!() );
            mask_pixels.push( mask_pix );
        }
    }
    let num_total = num_bad_col + num_bad_row + num_bad_both + num_dead_band + num_unknown;

    println!( "at line: {:?} : \n \
        num_bad_col   = {:?} \n \
        num_bad_row   = {:?} \n \
        num_bad_both  = {:?} \n \
        num_dead_band = {:?} \n \
        num_unknown   = {:?} \n \
        num_total     = {:?} \n" , 
           line!(), 
           num_bad_col,
           num_bad_row,
           num_bad_both,
           num_dead_band,
           num_unknown,
           num_total
       );
    Some ( mask_pixels )
}


pub fn to_diff_pair( file_set : &Vec<DirEntry> ) -> ( Option<Vec<Pixel> >, Option<Vec<Pixel> > ) {

    let open_test_files = file_set.iter().filter_map( | this_entry | {
        let this_entry_path = this_entry.path();
         if this_entry_path.to_str().unwrap().contains("C1717") 
         || this_entry_path.to_str().unwrap().contains("C2525") {
             Some (this_entry )
         } else {
                 None
         }
    } ).collect::<Vec<&DirEntry>>();
    
    let ( open_diff_pixels, marked_pixels )=
    { 
        let mut oit = open_test_files.iter();
        let lhs = oit.next().unwrap().path();
        let rhs = oit.next().unwrap().path(); 
        let open_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "open abs diff failed ");
        let ( marked_pixels_opt, bad_opens ) = mark_open_bads (0.3f32, &open_diff_pix );
        let marked_pixels = marked_pixels_opt.expect( "marking open bads failed for short test ");
        // let bad_opens = count_bad_pixels( 0.3f32, &open_diff_pix, &open_diff_pix );
        print!(" number of bad opens for 1717 - 2525 \n( {:?},\n- {:?} ) = {:?}\n", lhs, rhs, bad_opens );
        ( open_diff_pix, marked_pixels )
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
        let mut sit = short_test_files.iter();
        let lhs = sit.next().unwrap().path();
        let rhs = sit.next().unwrap().path(); 
        let short_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "short abs diff failed ");
        
        let mask_for_shorts = pixels_to_mask( &marked_pixels, 1864, 1632 ).expect( " unable to create mask" );
        // let threshold_for_shorts  = 0.5f32;         
        // stack over flow!! using the constant above does not cause the SO
        let threshold_for_shorts  = collect_unmasked_pixel_values_median( &mask_for_shorts, &short_diff_pix ).expect(" unable to collect unmasked pixels");
        
        let bad_shorts = count_bad_pixels( threshold_for_shorts, &mask_for_shorts, &short_diff_pix );
        // println!( "at line: {:?} ", line!() );
        print!(" number of bad shorts for 2517 - 1725 \n( {:?},\n- {:?} ) = {:?} <==> Threshold = {:?}\n", lhs, rhs, bad_shorts, threshold_for_shorts );
        short_diff_pix
    };
    
    ( Some( open_diff_pixels ) , Some( short_diff_pixels ) )
}
