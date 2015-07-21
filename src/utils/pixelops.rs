
use image::other::{
    BadType,
    Pixel
};

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

pub fn make_pixel_u16( (i, val ) : ( usize, &u16 ) ) -> Pixel {
    if is_dead_band( i, 1864, 1632 ).unwrap() {
        Pixel { value : val.clone() as f32, valid : BadType::DeadBand }
    }  else
    {
        Pixel { value : val.clone() as f32, valid : BadType::Unknown }
    }
}


pub fn make_pixel_f32( (i, val ) : ( usize, &f32 ) ) -> Pixel {
    if is_dead_band( i, 1864, 1632 ).expect( "is dead band paniced" ) {
        Pixel { value : val.clone() as f32, valid : BadType::DeadBand }
    }  else
    {
        Pixel { value : val.clone() as f32, valid : BadType::Unknown }
    }
}
