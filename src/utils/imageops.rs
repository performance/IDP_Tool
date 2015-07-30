use std::fs::{DirEntry};
use std::cmp::Ordering;
use image::other::{
    BadType,
    Pixel,
    ShortDiagonalStats
};

use utils::file::{
    absolute_difference_of_IDP_Imges
};

use super::dimensions::{WIDTH, HEIGHT };

fn mark_ignored_pixels( ps: &Vec<Pixel>, ignore_edges: usize ) ->  Option< Vec<Pixel> > {
    let total_pix = ps.len();
    // assert that this is equal to width times height
    
    let mut mask_pixels : Vec<Pixel> = Vec::with_capacity(total_pix); 
    let mut pit = ps.iter();
    
    for row in 0..HEIGHT {
        if row < ignore_edges || row >= ( HEIGHT - ignore_edges ) {
            for _col in 0..WIDTH {
                let ignored_pix   = pit.next().unwrap();
                let ignored_value = ignored_pix.value;
                mask_pixels.push( Pixel{ value : ignored_value, valid: BadType::Ignored } );
            }
        } else {
            for col in 0..WIDTH {
                let this_pix = pit.next().unwrap();
                let this_valid = this_pix.valid;
                let this_value = this_pix.value;

                if col < ignore_edges || col >= ( WIDTH - ignore_edges ) {
                    let mask_pix = if BadType::DeadBand == this_valid {
                        Pixel{ value : this_value, valid: this_valid  }
                    } else {
                        Pixel{ value : this_value, valid: BadType::Ignored } 
                    };
                    mask_pixels.push( mask_pix ); 
                }
                else {
                    let mask_pix = Pixel{ value : this_value, valid: this_valid  };
                    mask_pixels.push( mask_pix );
                }
            }
        }
    }
    Some ( mask_pixels )
}



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

fn apply_mask( ms: &Vec<Pixel>, ps: &Vec<Pixel> )  ->  ( Option<Vec<Pixel> > ) {
    // TODO: assert that they both have the same length or handle mismatch
    let mut masked_pixels : Vec<Pixel> = Vec::with_capacity( ps.len() );
    for ( m, p ) in ms.iter().zip( ps.iter() ) {
        let mask_pix = Pixel{ value: p.value, valid: m.valid };
        masked_pixels.push( mask_pix );
    }
    Some( masked_pixels )
}

fn mark_short_bads( threshold: f32, ps: &Vec<Pixel> )  ->  ( Option<Vec<Pixel> >, u64, usize, usize, usize ) {
    let total_pix = ps.len();
    let mut count = 0u64;
    let mut marked_pixels : Vec<Pixel> = Vec::with_capacity(total_pix);
    let mut sdstats: Vec< ShortDiagonalStats > =  Vec::with_capacity( WIDTH + HEIGHT );
    for _idx in 0..(WIDTH + HEIGHT) {
        sdstats.push( ShortDiagonalStats { number_of_pixels_measured: 0, number_of_bad_shorts: 0 } )
    }
    let mut pit = ps.iter();
    // for p in ps.iter() {
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            let _idx = row + col;
            let p = pit.next().unwrap();
            let mask_pix = 
                if p.valid == BadType::Unknown {
                    sdstats[ _idx ].number_of_pixels_measured += 1;
                    if threshold > p.value {
                        count += 1;
                        sdstats[ _idx ].number_of_bad_shorts += 1;
                        Pixel{ value: p.value, valid: BadType::ShortBad }
                    } else { Pixel{ value: p.value, valid: p.valid } }
                } else {
                    Pixel{ value: p.value, valid: p.valid }
                };
            marked_pixels.push( mask_pix );
        }
    }
    let short_bad_diagonals : Vec<bool> = sdstats.iter().map( | &ShortDiagonalStats { number_of_pixels_measured, number_of_bad_shorts } |  number_of_bad_shorts > ( number_of_pixels_measured / 2 ) ).collect::<Vec<bool>>();
    let number_of_bad_diagonals = short_bad_diagonals.iter().fold( 0, | sum, flag | sum + if *flag { 1 } else { 0 } );
    let number_of_short_bads_not_in_bad_diagonals = short_bad_diagonals.iter()
                                                       .zip( sdstats.iter() )
                                                       .fold( 0, |sum, ( flag, &ShortDiagonalStats { number_of_pixels_measured, number_of_bad_shorts } )| sum + if  *flag { 0 * number_of_pixels_measured } else { number_of_bad_shorts } );
    let ( number_of_adjacent_bad_diagonals, _ ) = short_bad_diagonals.iter()
                                                              .fold( ( 0, false ),  | ( sum, prev_diag_bad ), flag | if *flag && prev_diag_bad { ( sum+1, false ) } else { ( sum, *flag ) } );

    ( Some( marked_pixels ), count, number_of_short_bads_not_in_bad_diagonals as usize, number_of_bad_diagonals, number_of_adjacent_bad_diagonals )
}


fn _count_bad_pixels( threshold: f32, ms: &Vec<Pixel>, ps: &Vec<Pixel> ) -> u64 {
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
        // println!( "unmasked_pixel_values.len() = {:?}", num_elems ); 
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
fn pixels_to_mask( ps: &Vec<Pixel>) ->  Option<( Vec<Pixel>, u64, u64, u64, usize, usize, u64 ) >{
    let total_pix = ps.len();
    // assert that this is equal to width times height
    
    let mut bad_pix_in_row : Vec<usize> = Vec::with_capacity( HEIGHT );
    let mut bad_pix_in_col : Vec<usize> = Vec::with_capacity( WIDTH );
    for _row in 0..HEIGHT {
        bad_pix_in_row.push( 0 );
    }
    for _col in 0..WIDTH {
        bad_pix_in_col.push( 0 );
    }

    {
        let mut pit = ps.iter();
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if BadType::OpenBad == pit.next().unwrap().valid {
                        bad_pix_in_col[ col ] += 1;
                        bad_pix_in_row[ row ] += 1;
                }
            }
        }
    }

    let bad_cols : Vec<bool> = bad_pix_in_col.iter().map( |col_count| *col_count > ( HEIGHT / 2 ) ).collect();
    let bad_rows : Vec<bool> = bad_pix_in_row.iter().map( |row_count| *row_count > ( WIDTH  / 2 ) ).collect();

    let number_of_open_bads_in_bad_cols = 
        bad_cols.iter().zip( bad_pix_in_col.iter() ).fold( 0, |sum, ( flag, count ) | sum + if *flag { *count } else { 0 } );

    let number_of_open_bads_in_bad_rows = 
        bad_rows.iter().zip( bad_pix_in_row.iter() ).fold( 0, |sum, ( flag, count ) | sum + if *flag { *count } else { 0 } );


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
    let mut num_ignored = 0u64; 
    
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            let this_pix = pit.next().unwrap();
            let this_valid = this_pix.valid;
            let this_value = this_pix.value;

            let mask_pix = match this_valid {
                BadType::DeadBand => { num_dead_band += 1; Pixel{ value : this_value, valid: this_valid  } },
                BadType::Ignored  => { num_ignored   += 1; Pixel{ value : this_value, valid: this_valid  } },
                _ => {
                    match ( bad_cols[ col ], bad_rows[ row ] ) {
                        ( true,  true  ) => { num_bad_both += 1; Pixel{ value : 999f32,     valid: BadType::OpenBadBoth } },
                        ( true,  false ) => { num_bad_col  += 1; Pixel{ value : 999f32,     valid: BadType::OpenBadCol  } },
                        ( false, true  ) => { num_bad_row  += 1; Pixel{ value : 999f32,     valid: BadType::OpenBadRow  } },
                        ( false, false ) => { num_unknown  += 1; Pixel{ value : this_value, valid: this_valid           } },
                    }
                }
            };
            mask_pixels.push( mask_pix );
        }
    }
    let _num_total = num_bad_col + num_bad_row + num_bad_both + num_dead_band + num_ignored + num_unknown;
    let num_measured = total_pix as u64 - ( num_ignored + num_dead_band );

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
    //        _num_total
    //    );
    Some ( ( mask_pixels, number_of_bad_columns as u64, number_of_bad_rows as u64, num_measured as u64, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, num_unknown ) )
}

// TODO: use a struct to get the measurement data out
pub fn to_diff_pair( file_set : &Vec<DirEntry>, open_threshold: f32, short_threshold: f32, ignore_edges : usize ) -> 
    ( ( Option<Vec<Pixel> >, Option<Vec<Pixel> > ),
      ( u64, u64, u64, u64, f32 , u64, usize, usize, usize, usize, usize ) 
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

    // println!("open_test_files.len() = {:?}", &open_test_files.len() );

    // TODO: make sure there are at least two files. exactly two?
    
    let ( open_diff_pixels, mask_for_shorts, bad_opens, number_of_bad_columns, number_of_bad_rows, num_total, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, num_unknown ) =
    { 
        let mut oit = open_test_files.iter();
        let lhs = oit.next().unwrap().path();
        let rhs = oit.next().unwrap().path(); 
        let open_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "open abs diff failed ");
        let ig_marked_pixels = mark_ignored_pixels( &open_diff_pix, ignore_edges ).expect(" could not mark ignored pixels");
        let ( marked_pixels_opt, bad_opens ) = mark_open_bads ( open_threshold, &ig_marked_pixels );
        let marked_pixels = marked_pixels_opt.expect( "marking open bads failed for open test ");
        let ( mask_for_shorts, number_of_bad_columns, number_of_bad_rows, num_total, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, num_unknown ) 
                = pixels_to_mask( &marked_pixels ).expect( " unable to create mask" );
        // print!(" number of bad opens for 1717 - 2525 \n( {:?},\n- {:?} ) = {:?}\n", lhs, rhs, bad_opens );
        ( open_diff_pix, mask_for_shorts, bad_opens, number_of_bad_columns, number_of_bad_rows, num_total, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, num_unknown )
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

    // TODO: make sure there are at least two files. exactly two?

    let ( short_diff_pixels, bad_shorts, threshold_for_shorts, number_of_short_bads_not_in_bad_diagonals, number_of_bad_diagonals, number_of_adjacent_bad_diagonals ) =
    { 
        if num_unknown > 0  
        {
            let mut sit = short_test_files.iter();
            let lhs = sit.next().unwrap().path();
            let rhs = sit.next().unwrap().path(); 
            // println!(" short_test_files:\n {:?},\n {:?} ", &lhs, &rhs );
            let short_diff_pix = absolute_difference_of_IDP_Imges( &lhs, &rhs ).expect( "short abs diff failed ");
            let threshold_for_shorts  = short_threshold * median_of_unmasked_pixel_values( &mask_for_shorts, &short_diff_pix ).expect(" unable to collect unmasked pixels");
            
            let masked_short_diff_pix = apply_mask( &mask_for_shorts, &short_diff_pix ).expect(" Unable to apply openmask to short diffs " );
            let ( _marked_short_pixels_opt, num_bad_shorts, number_of_short_bads_not_in_bad_diagonals, number_of_bad_diagonals, number_of_adjacent_bad_diagonals ) = 
                mark_short_bads( threshold_for_shorts, &masked_short_diff_pix );
            ( short_diff_pix, num_bad_shorts, threshold_for_shorts, number_of_short_bads_not_in_bad_diagonals, number_of_bad_diagonals, number_of_adjacent_bad_diagonals )
        } else {
            ( vec![], 0u64, 0.0f32, 0usize, 0usize, 0usize )
        }

    };
    (
        ( Some( open_diff_pixels ) , Some( short_diff_pixels ) ),
        ( bad_opens, number_of_bad_columns, number_of_bad_rows, bad_shorts, threshold_for_shorts , num_total, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, number_of_short_bads_not_in_bad_diagonals, number_of_bad_diagonals, number_of_adjacent_bad_diagonals )

    )
}

