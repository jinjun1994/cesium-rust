use cesium_math::{write_text_to_canvas, CanvasTextResult, Color, WriteTextOptions};

fn get_color_change_count(canvas: &CanvasTextResult, row: u32) -> u32 {
    let mut color_change_count = 0;
    let pixel = canvas.get_pixel(0, row);
    let mut last_red = if pixel[0] > 0 && pixel[0] != 255 { 255 } else { pixel[0] };
    let mut last_green = if pixel[1] > 0 && pixel[1] != 255 { 255 } else { pixel[1] };
    let mut last_blue = if pixel[2] > 0 && pixel[2] != 255 { 255 } else { pixel[2] };

    for col in 0..canvas.width {
        let p = canvas.get_pixel(col, row);
        let mut red = p[0];
        let mut green = p[1];
        let mut blue = p[2];

        if red > 0 && red != 255 {
            red = 255;
        }
        if green > 0 && green != 255 {
            green = 255;
        }
        if blue > 0 && blue != 255 {
            blue = 255;
        }

        if red != last_red || green != last_green || blue != last_blue {
            color_change_count += 1;
        }
        last_red = red;
        last_green = green;
        last_blue = blue;
    }

    color_change_count
}

#[test]
fn test_returns_none_when_text_is_blank() {
    let canvas = write_text_to_canvas(Some(""), None).unwrap();
    assert!(canvas.is_none());
}

#[test]
fn test_throws_when_text_is_undefined() {
    let result = write_text_to_canvas(None, None);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(format!("{:?}", err).contains("text is required"));
}

#[test]
fn test_sizes_the_canvas_to_fit_the_text() {
    let canvas1 = write_text_to_canvas(Some("m"), None).unwrap().unwrap();
    let canvas2 = write_text_to_canvas(Some("mm"), None).unwrap().unwrap();

    assert_ne!(canvas1.width, canvas2.width);
    assert_eq!(canvas1.height, canvas2.height);
}

#[test]
fn test_allows_the_text_to_be_either_stroked_or_filled() {
    let row = 20;

    let canvas1 = write_text_to_canvas(
        Some("I"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            fill: true,
            fill_color: Color::RED,
            stroke: false,
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    let count1 = get_color_change_count(&canvas1, row);
    assert!(count1 == 1 || count1 == 2, "count1 should be 1 or 2, got {}", count1);

    let canvas2 = write_text_to_canvas(
        Some("I"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            fill: false,
            stroke: true,
            stroke_color: Color::BLUE,
            stroke_width: 2.0,
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    let count2 = get_color_change_count(&canvas2, row);
    assert!(count2 == 3 || count2 == 4, "count2 should be 3 or 4, got {}", count2);
}

#[test]
fn test_background_color_defaults_to_transparent() {
    let canvas = write_text_to_canvas(
        Some("a"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    let pixel = canvas.get_pixel(0, 0);
    assert_eq!(pixel, [0, 0, 0, 0]);
}

#[test]
fn test_background_can_be_set() {
    let canvas = write_text_to_canvas(
        Some("a"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            background_color: Color::RED,
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    let pixel = canvas.get_pixel(0, 0);
    assert_eq!(pixel, [255, 0, 0, 255]);
}

#[test]
fn test_border_can_be_set() {
    let canvas1 = write_text_to_canvas(
        Some("a"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            background_color: Color::RED,
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    let canvas2 = write_text_to_canvas(
        Some("a"),
        Some(WriteTextOptions {
            font: "90px Open Sans".to_string(),
            background_color: Color::RED,
            padding: 2.0,
            ..Default::default()
        }),
    )
    .unwrap()
    .unwrap();

    assert_eq!(canvas2.width, canvas1.width + 4);
    assert_eq!(canvas2.height, canvas1.height + 4);
}
