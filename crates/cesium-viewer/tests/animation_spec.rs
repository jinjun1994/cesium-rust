use cesium_viewer::widgets::{Animation, AnimationContainer, AnimationViewModel, ClockViewModel};

#[test]
fn can_create_and_destroy() {
    let clock_view_model = ClockViewModel::default();
    let animation_view_model = AnimationViewModel::new(clock_view_model);
    let animation = Animation::new("document.body", animation_view_model);

    assert_eq!(animation.container().id, "document.body");
    assert!(animation.container().in_dom);
    assert!(!animation.is_destroyed());

    animation.destroy();
    assert!(animation.is_destroyed());
}

#[test]
fn can_create_with_container_not_in_the_dom() {
    let mut container = AnimationContainer::new("detached_div", false);
    let clock_view_model = ClockViewModel::default();
    let animation_view_model = AnimationViewModel::new(clock_view_model);
    let animation = Animation::new(container.clone(), animation_view_model);

    assert!(!animation.container().in_dom);
    assert_eq!(animation.apply_theme_changes_count(), 0);

    // Simulate appending container to DOM and triggering theme changes
    container.in_dom = true;
    animation.apply_theme_changes();

    assert_eq!(animation.apply_theme_changes_count(), 1);
    animation.destroy();
    assert!(animation.is_destroyed());
}

#[test]
fn can_destroy_without_container_ever_being_in_the_dom() {
    let container = AnimationContainer::new("detached_div", false);
    let clock_view_model = ClockViewModel::default();
    let animation_view_model = AnimationViewModel::new(clock_view_model);
    let animation = Animation::new(container, animation_view_model);

    assert!(!animation.container().in_dom);
    animation.destroy();
    assert!(animation.is_destroyed());
}
