use photo::ImageRGB;

const INPUT_DIR: &str = "input";
const IMAGE_NAME: &str = "colour-u8.png";

fn main() {
    let filepath = format!("{}/{}", INPUT_DIR, IMAGE_NAME);
    let image = ImageRGB::<u8>::load(filepath).expect("Failed to load image");
    println!("{}", image);
}
