use agb_image_converter::{convert_image, Colour, ImageConverterConfig, TileSize};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    convert_image(
        ImageConverterConfig::builder()
            .tile_size(TileSize::Tile8)
            .transparent_colour(Colour::from_rgb(44, 232, 244))
            .input_image("gfx/object_sheet.png".into())
            .output_file(format!("{}/object_sheet.rs", out_dir).into())
            .build(),
    );
}
