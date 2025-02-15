use photo::ImageGA;

const INPUT_DIR: &str = "input";
const IMAGE_NAME: &str = "grayscale_alpha-f64.png";

fn main() {
    let filepath = format!("{}/{}", INPUT_DIR, IMAGE_NAME);
    let image = ImageGA::<f64>::load(filepath).expect("Failed to load image");
    println!("{}", image);
}
