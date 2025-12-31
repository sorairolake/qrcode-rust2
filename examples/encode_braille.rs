use qrcode2::{QrCode, render::braille};

fn main() {
    let code = QrCode::new(b"Hello").unwrap();
    let string = code.render::<braille::BraillePixel>().quiet_zone(0).build();
    println!("{string}");
}
