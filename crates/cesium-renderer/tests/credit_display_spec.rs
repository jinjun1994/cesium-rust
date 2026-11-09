use cesium_math::Credit;
use cesium_renderer::{CreditDisplay, DEFAULT_CREDIT_DELIMITER};

#[test]
fn test_default_constructor() {
    let display = CreditDisplay::default();
    assert_eq!(display.delimiter, DEFAULT_CREDIT_DELIMITER);
    assert!(display.screen_credits().is_empty());
    assert!(display.lightbox_credits().is_empty());
    assert_eq!(display.formatted_screen_credits(), "");
    assert_eq!(display.formatted_lightbox_credits(), "");
    assert!(!display.is_destroyed());
}

#[test]
fn test_credits_identity_and_clone() {
    let credit1 = Credit::new("http://cesiumjs.org/credit1", Some(true));
    let credit2 = Credit::new("http://cesiumjs.org/credit2", Some(true));
    let credit3 = Credit::new("http://cesiumjs.org/credit1", Some(true));

    assert_ne!(credit1, credit2);
    assert_eq!(credit1, credit3);

    let credit1_clone = credit1.clone();
    assert_eq!(credit1, credit1_clone);
}

#[test]
fn test_displays_credit() {
    let mut display = CreditDisplay::default();
    let credit = Credit::new("<a href=\"http://cesiumjs.org\">CesiumJS.org</a>", Some(true));

    display.begin_frame();
    display.add_credit_to_next_frame(credit.clone()).unwrap();
    display.end_frame();

    assert_eq!(display.screen_credits().len(), 1);
    assert_eq!(display.screen_credits()[0], &credit);
}

#[test]
fn test_updates_when_credits_change_across_frames() {
    let credit1 = Credit::new("credit1", Some(true));
    let credit2 = Credit::new("credit2", Some(true));

    let mut display = CreditDisplay::default();

    // Frame 1: only credit1
    display.begin_frame();
    display.add_credit_to_next_frame(credit1.clone()).unwrap();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "credit1");

    // Frame 2: only credit2
    display.begin_frame();
    display.add_credit_to_next_frame(credit2.clone()).unwrap();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "credit2");

    // Frame 3: both
    display.begin_frame();
    display.add_credit_to_next_frame(credit1.clone()).unwrap();
    display.add_credit_to_next_frame(credit2.clone()).unwrap();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "credit1, credit2");

    // Frame 4: neither
    display.begin_frame();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "");
}

#[test]
fn test_custom_delimiter() {
    let credit1 = Credit::new("credit1", Some(true));
    let credit2 = Credit::new("credit2", Some(true));
    let credit3 = Credit::new("credit3", Some(true));

    let mut display = CreditDisplay::new(Some(" | ".to_string()));
    display.begin_frame();
    display.add_credit_to_next_frame(credit1).unwrap();
    display.add_credit_to_next_frame(credit2).unwrap();
    display.add_credit_to_next_frame(credit3).unwrap();
    display.end_frame();

    assert_eq!(display.formatted_screen_credits(), "credit1 | credit2 | credit3");
}

#[test]
fn test_static_credits() {
    let static_credit = Credit::new("Default Static Credit", Some(true));
    let dynamic_credit = Credit::new("Dynamic Credit", Some(true));

    let mut display = CreditDisplay::default();
    display.add_static_credit(static_credit.clone()).unwrap();

    // Frame 1: static + dynamic
    display.begin_frame();
    display.add_credit_to_next_frame(dynamic_credit.clone()).unwrap();
    display.end_frame();
    assert_eq!(
        display.formatted_screen_credits(),
        "Default Static Credit, Dynamic Credit"
    );

    // Frame 2: dynamic cleared, static persists
    display.begin_frame();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "Default Static Credit");

    // Remove static
    assert!(display.remove_static_credit(&static_credit));
    display.begin_frame();
    display.end_frame();
    assert_eq!(display.formatted_screen_credits(), "");

    // Remove again returns false
    assert!(!display.remove_static_credit(&static_credit));
}

#[test]
fn test_distinguishes_screen_and_lightbox_credits() {
    let screen = Credit::new("Screen Credit", Some(true));
    let lightbox = Credit::new("Lightbox Credit", Some(false));

    let mut display = CreditDisplay::default();
    display.begin_frame();
    display.add_credit_to_next_frame(screen.clone()).unwrap();
    display.add_credit_to_next_frame(lightbox.clone()).unwrap();
    display.end_frame();

    assert_eq!(display.screen_credits(), vec![&screen]);
    assert_eq!(display.lightbox_credits(), vec![&lightbox]);
    assert_eq!(display.formatted_screen_credits(), "Screen Credit");
    assert_eq!(display.formatted_lightbox_credits(), "Lightbox Credit");
}

#[test]
fn test_destroy() {
    let mut display = CreditDisplay::default();
    assert!(!display.is_destroyed());
    display.destroy();
    assert!(display.is_destroyed());

    let credit = Credit::new("Credit", Some(true));
    assert!(display.add_credit_to_next_frame(credit.clone()).is_err());
    assert!(display.add_static_credit(credit.clone()).is_err());
    assert!(!display.remove_static_credit(&credit));
}
