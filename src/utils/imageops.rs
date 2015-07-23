use std::fs::{DirEntry};
use std::cmp::Ordering;
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

fn median_of_unmasked_pixel_values ( ms: &Vec<Pixel>, ps: &Vec<Pixel> ) -> Option<f32> {

    let threshold_for_shorts  = {
        let mut unmasked_pixel_values: Vec<f32> = ms.iter().zip( ps.iter() ).filter_map( 
            | ( m,p ) | if m.valid == BadType::Unknown { Some( p.value ) } else { None } 
        ).collect();
        let num_elems = unmasked_pixel_values.len();
        let mid_elem = num_elems / 2;
        let mid_elem_prev = mid_elem - 1;
        unmasked_pixel_values.sort_by(| a, b | if a < b { Ordering::Less } else if  a > b { Ordering::Greater } else { Ordering::Equal } );
        if ( num_elems % 2 ) == 1  { 
           ( unmasked_pixel_values[ mid_elem ] + unmasked_pixel_values[ mid_elem_prev ] ) / 2.0
        } else
        {
            unmasked_pixel_values[ mid_elem ]
        }
    };
    Some( threshold_for_shorts )
}


// masks out all the columns and rows with > 50% bad pixels in them
// TODO: use a struct to return stuff 
fn pixels_to_mask( ps: &Vec<Pixel>,  width: usize, height: usize  ) ->  Option<( Vec<Pixel>, u64, u64, u64, u64 ) >{
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

    let bad_cols : Vec<bool> = bad_pix_in_col.iter().map( |col_count| *col_count > ( height / 2 ) ).collect();
    let bad_rows : Vec<bool> = bad_pix_in_row.iter().map( |row_count| *row_count > ( width  / 2 ) ).collect();

    let number_of_bad_columns = bad_cols.iter().fold( 0, |sum, x | sum + if *x { 1 } else {0 } );
    let number_of_bad_rows    = bad_rows.iter().filter( | &x | *x ).collect::<Vec<_>>().len(); // | fold( 0, |sum, x | sum + if *x { 1 } else {0 } );
    
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
        for col in 0..width {
            let this_pix = pit.next().unwrap();
            let this_valid = this_pix.valid;
            let this_value = this_pix.value;

            let mask_pix = if BadType::DeadBand == this_valid {
                    num_dead_band += 1;
                    Pixel{ value : this_value, valid: this_valid  }
                } else {

                    match ( bad_cols[ col ], bad_rows[ row ] ) {
                        ( true,  true  ) => { num_bad_both += 1; Pixel{ value : 999f32, valid: BadType::OpenBadBoth } },
                        ( true,  false ) => { num_bad_col  += 1; Pixel{ value : 999f32, valid: BadType::OpenBadCol  } },
                        ( false, true  ) => { num_bad_row  += 1; Pixel{ value : 999f32, valid: BadType::OpenBadRow  } },
                        ( false, false ) => {
                            num_unknown += 1;
                            Pixel{ value : this_value, valid: this_valid  }
                        },
                    }
                };
            mask_pixels.push( mask_pix );
        }
    }
    let num_total = num_bad_col + num_bad_row + num_bad_both + num_dead_band + num_unknown;

    // println!( " \
    //     number of bad columns             = {:?} \n \
    //     number of bad rows                = {:?} \n \
    //     number of pixels in bad columns   = {:?} \n \
    //     number of pixels in bad rows      = {:?} \n \
    //     number o  pixels in bad both      = {:?} \n \
    //     num_dead_band = {:?} \n \
    //     num_unknown   = {:?} \n \
    //     num_total     = {:?} \n" ,
    //        number_of_bad_columns,
    //        number_of_bad_rows,
    //        num_bad_col,
    //        num_bad_row,
    //        num_bad_both,
    //        num_dead_band,
    //        num_unknown,
    //        num_total
    //    );
    Some ( ( mask_pixels, number_of_bad_columns as u64, number_of_bad_rows as u64, num_total, num_dead_band ) )
}


pub fn to_diff_pair( file_set : &Vec<DirEntry>, open_threshold: f32 ) -> 
    ( ( Option<Vec<Pixel> >, Option<Vec<Pixel> > ),
      ( u64, u64, u64, u64, f32 , u64, u64 ) 
    ) {

    let open_test_files = file_set.iter().filter_map( | this_entry | {
        let this_entry_path = this_entry.path();
         if this_entry_path.to_str().unwrap().contains("C1717") 
         || this_entry_path.to_str().unwrap().contains("C2525") {
             Some (this_entry )
         } else {
                 None
         }
    } ).collect::<Vec<&DirEntry>>();
    
    let ( open_diff_pixels, mask_for_shorts, bad_opens, number_of_bad_columns, number_of_bad_rows, num_total, num_dead_band )=
    { 
        let mut oit = open_test_files.iter();
        let lhs = oit.next().unwrap().path();
        let rhs = oit.next().unwrap().path(); 
        let open_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "open abs diff failed ");
        let ( marked_pixels_opt, bad_opens ) = mark_open_bads ( open_threshold, &open_diff_pix );
        let marked_pixels = marked_pixels_opt.expect( "marking open bads failed for short test ");
        let ( mask_for_shorts, number_of_bad_columns, number_of_bad_rows, num_total, num_dead_band ) 
                = pixels_to_mask( &marked_pixels, 1864, 1632 ).expect( " unable to create mask" );
        // print!(" number of bad opens for 1717 - 2525 \n( {:?},\n- {:?} ) = {:?}\n", lhs, rhs, bad_opens );
        ( open_diff_pix, mask_for_shorts, bad_opens, number_of_bad_columns, number_of_bad_rows, num_total, num_dead_band )
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
    
    let ( short_diff_pixels, bad_shorts, threshold_for_shorts ) =
    { 
        let mut sit = short_test_files.iter();
        let lhs = sit.next().unwrap().path();
        let rhs = sit.next().unwrap().path(); 
        let short_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "short abs diff failed ");
        

        // let threshold_for_shorts  = 0.5f32;         
        // stack over flow!! using the constant above does not cause the SO
        let threshold_for_shorts  = median_of_unmasked_pixel_values( &mask_for_shorts, &short_diff_pix ).expect(" unable to collect unmasked pixels");
        
        let bad_shorts = count_bad_pixels( threshold_for_shorts, &mask_for_shorts, &short_diff_pix );
        // println!( "at line: {:?} ", line!() );
        // print!(" number of bad shorts for 2517 - 1725 \n( {:?},\n- {:?} ) = {:?} <==> Threshold = {:?}\n", lhs, rhs, bad_shorts, threshold_for_shorts );
        ( short_diff_pix, bad_shorts, threshold_for_shorts )
    };
    (
        ( Some( open_diff_pixels ) , Some( short_diff_pixels ) ),
        ( bad_opens, number_of_bad_columns, number_of_bad_rows, bad_shorts, threshold_for_shorts , num_total, num_dead_band )

    )
}
