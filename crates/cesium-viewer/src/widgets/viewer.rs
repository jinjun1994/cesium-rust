//! Viewer widget matching CesiumJS `Widgets/Viewer/Viewer.js` (Apache-2.0).

use cesium_math::{Clock, DeveloperError};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::animation::{Animation, AnimationContainer};
use super::animation_view_model::AnimationViewModel;
use super::base_layer_picker::{BaseLayerPicker, BaseLayerPickerContainer};
use super::clock_view_model::ClockViewModel;
use super::fullscreen_button::{FullscreenButton, FullscreenButtonContainer};
use super::geocoder::{Geocoder, GeocoderContainer};
use super::home_button::{HomeButton, HomeButtonContainer};
use super::info_box::{InfoBox, InfoBoxContainer};
use super::navigation_help_button::{
    NavigationHelpButton, NavigationHelpButtonContainer, NavigationHelpButtonOptions,
};
use super::performance_watchdog::{PerformanceWatchdog, PerformanceWatchdogContainer, PerformanceWatchdogOptions};
use super::performance_watchdog_view_model::WatchdogScene;
use super::scene_mode_picker::{SceneModePicker, SceneModePickerContainer};
use super::scene_mode_picker_view_model::SceneModePickerScene;
use super::selection_indicator::SelectionIndicator;
use super::selection_indicator_view_model::{SelectionIndicatorContainer, SelectionIndicatorScene};
use super::timeline::{Timeline, TimelineContainer};
use super::vr_button::{VRButton, VRButtonContainer};
use super::vr_button_view_model::VRScene;

/// Abstract UI container representation for `Viewer`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewerContainer {
    pub id: String,
    pub exists: bool,
}

impl ViewerContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// Options for configuring `Viewer`.
#[derive(Clone, Debug)]
pub struct ViewerOptions {
    pub animation: bool,
    pub base_layer_picker: bool,
    pub fullscreen_button: bool,
    pub vr_button: bool,
    pub geocoder: bool,
    pub home_button: bool,
    pub info_box: bool,
    pub scene_mode_picker: bool,
    pub selection_indicator: bool,
    pub timeline: bool,
    pub navigation_help_button: bool,
    pub navigation_instructions_initially_visible: bool,
    pub scene3d_only: bool,
    pub should_animate: bool,
    pub clock_view_model: Option<ClockViewModel>,
    pub use_browser_recommended_resolution: bool,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self {
            animation: true,
            base_layer_picker: true,
            fullscreen_button: true,
            vr_button: false,
            geocoder: true,
            home_button: true,
            info_box: true,
            scene_mode_picker: true,
            selection_indicator: true,
            timeline: true,
            navigation_help_button: true,
            navigation_instructions_initially_visible: false,
            scene3d_only: false,
            should_animate: false,
            clock_view_model: None,
            use_browser_recommended_resolution: true,
        }
    }
}

/// Drag and drop mixin options for `Viewer`.
#[derive(Clone, Debug)]
pub struct ViewerDragDropOptions {
    pub drop_target: Option<String>,
    pub clear_on_drop: bool,
    pub clamp_to_ground: bool,
    pub fly_to_on_drop: bool,
}

impl Default for ViewerDragDropOptions {
    fn default() -> Self {
        Self {
            drop_target: None,
            clear_on_drop: true,
            clamp_to_ground: true,
            fly_to_on_drop: true,
        }
    }
}

/// Performance watchdog mixin options for `Viewer`.
#[derive(Clone, Debug, Default)]
pub struct ViewerPerformanceWatchdogOptions {
    pub low_frame_rate_message: Option<String>,
}

/// The main `Viewer` widget coordinating all Cesium sub-widgets.
pub struct Viewer {
    container: ViewerContainer,
    clock: Rc<RefCell<Clock>>,
    clock_view_model: ClockViewModel,
    animation: Option<Animation>,
    base_layer_picker: Option<BaseLayerPicker>,
    fullscreen_button: Option<FullscreenButton>,
    vr_button: Option<VRButton>,
    geocoder: Option<Geocoder>,
    home_button: Option<HomeButton>,
    info_box: Option<InfoBox>,
    scene_mode_picker: Option<SceneModePicker>,
    selection_indicator: Option<SelectionIndicator>,
    timeline: Option<Timeline>,
    navigation_help_button: Option<NavigationHelpButton>,
    performance_watchdog: Option<PerformanceWatchdog>,
    pub drop_target: Option<String>,
    pub drop_enabled: bool,
    pub clear_on_drop: bool,
    pub clamp_to_ground: bool,
    pub fly_to_on_drop: bool,
    pub use_browser_recommended_resolution: bool,
    is_destroyed: Cell<bool>,
}

impl Viewer {
    /// Creates a new `Viewer`.
    pub fn new(
        container: Option<ViewerContainer>,
        options: Option<ViewerOptions>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let opts = options.unwrap_or_default();

        let clock_view_model = match opts.clock_view_model {
            Some(cvm) => {
                if opts.should_animate {
                    cvm.clock().borrow_mut().set_should_animate(true);
                }
                cvm
            }
            None => {
                let clk = Rc::new(RefCell::new(Clock::default()));
                if opts.should_animate {
                    clk.borrow_mut().set_should_animate(true);
                }
                ClockViewModel::new(Some(clk))
            }
        };

        let clock = clock_view_model.clock();

        let animation = if opts.animation {
            let anim_container = AnimationContainer::new(format!("{}_anim", container.id), true);
            let anim_vm = AnimationViewModel::new(clock_view_model.clone());
            Some(Animation::new(anim_container, anim_vm))
        } else {
            None
        };

        let base_layer_picker = if opts.base_layer_picker {
            let blp_container = BaseLayerPickerContainer::new(format!("{}_blp", container.id), true);
            let globe = Rc::new(RefCell::new(super::base_layer_picker_view_model::StandaloneGlobe::default()));
            let blp_opts = super::base_layer_picker_view_model::BaseLayerPickerViewModelOptions {
                globe: Some(globe),
                ..Default::default()
            };
            Some(BaseLayerPicker::new(Some(blp_container), Some(blp_opts))?)
        } else {
            None
        };

        let fullscreen_button = if opts.fullscreen_button {
            let fs_container = FullscreenButtonContainer::new(format!("{}_fs", container.id), true);
            Some(FullscreenButton::new(Some(fs_container), None::<&str>)?)
        } else {
            None
        };

        let vr_button = if opts.vr_button {
            let vr_container = VRButtonContainer::new(format!("{}_vr", container.id), true);
            let vr_scene = Rc::new(RefCell::new(VRScene::new()));
            Some(VRButton::new(Some(vr_container), Some(vr_scene), None::<&str>)?)
        } else {
            None
        };

        let geocoder = if opts.geocoder {
            let geo_container = GeocoderContainer::new(format!("{}_geo", container.id), true);
            let geo_opts = super::geocoder_view_model::GeocoderViewModelOptions {
                scene_id: Some("scene_0".to_string()),
                flight_duration: None,
                geocoder_services: Vec::new(),
                destination_found: None,
            };
            Some(Geocoder::new(Some(geo_container), Some(geo_opts))?)
        } else {
            None
        };

        let home_button = if opts.home_button {
            let hb_container = HomeButtonContainer::new(format!("{}_hb", container.id), true);
            Some(HomeButton::new(Some(hb_container), Some("scene_0"), None)?)
        } else {
            None
        };

        let info_box = if opts.info_box {
            let ib_container = Rc::new(RefCell::new(InfoBoxContainer::new(format!("{}_ib", container.id))));
            Some(InfoBox::new(Some(ib_container))?)
        } else {
            None
        };

        let scene_mode_picker = if opts.scene_mode_picker {
            let smp_container = Rc::new(RefCell::new(SceneModePickerContainer::new(format!("{}_smp", container.id))));
            let smp_scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
            Some(SceneModePicker::new(Some(smp_container), Some(smp_scene), Some(0.0))?)
        } else {
            None
        };

        let selection_indicator = if opts.selection_indicator {
            let si_container = Rc::new(RefCell::new(SelectionIndicatorContainer::new(format!("{}_si", container.id))));
            let si_scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
            Some(SelectionIndicator::new(Some(si_container), Some(si_scene))?)
        } else {
            None
        };

        let timeline = if opts.timeline {
            let tl_container = Rc::new(RefCell::new(TimelineContainer::new(format!("{}_tl", container.id))));
            Some(Timeline::new(Some(tl_container), Some(Rc::clone(&clock)))?)
        } else {
            None
        };

        let navigation_help_button = if opts.navigation_help_button {
            let nhb_container = Rc::new(RefCell::new(NavigationHelpButtonContainer::new(format!("{}_nhb", container.id))));
            let nhb_opts = NavigationHelpButtonOptions {
                container: Some(nhb_container),
                instructions_initially_visible: Some(opts.navigation_instructions_initially_visible),
            };
            Some(NavigationHelpButton::new(Some(nhb_opts))?)
        } else {
            None
        };

        Ok(Self {
            container,
            clock,
            clock_view_model,
            animation,
            base_layer_picker,
            fullscreen_button,
            vr_button,
            geocoder,
            home_button,
            info_box,
            scene_mode_picker,
            selection_indicator,
            timeline,
            navigation_help_button,
            performance_watchdog: None,
            drop_target: None,
            drop_enabled: false,
            clear_on_drop: true,
            clamp_to_ground: true,
            fly_to_on_drop: true,
            use_browser_recommended_resolution: opts.use_browser_recommended_resolution,
            is_destroyed: Cell::new(false),
        })
    }

    /// Creates a `Viewer` from container ID string.
    pub fn from_id(
        container_id: &str,
        options: Option<ViewerOptions>,
        lookup: impl Fn(&str) -> Option<ViewerContainer>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), options)
    }

    /// Gets container.
    pub fn container(&self) -> &ViewerContainer {
        &self.container
    }

    /// Gets Clock.
    pub fn clock(&self) -> Rc<RefCell<Clock>> {
        Rc::clone(&self.clock)
    }

    /// Gets ClockViewModel.
    pub fn clock_view_model(&self) -> &ClockViewModel {
        &self.clock_view_model
    }

    /// Gets Animation widget.
    pub fn animation(&self) -> Option<&Animation> {
        self.animation.as_ref()
    }

    /// Gets BaseLayerPicker widget.
    pub fn base_layer_picker(&self) -> Option<&BaseLayerPicker> {
        self.base_layer_picker.as_ref()
    }

    /// Gets FullscreenButton widget.
    pub fn fullscreen_button(&self) -> Option<&FullscreenButton> {
        self.fullscreen_button.as_ref()
    }

    /// Gets VRButton widget.
    pub fn vr_button(&self) -> Option<&VRButton> {
        self.vr_button.as_ref()
    }

    /// Gets Geocoder widget.
    pub fn geocoder(&self) -> Option<&Geocoder> {
        self.geocoder.as_ref()
    }

    /// Gets HomeButton widget.
    pub fn home_button(&self) -> Option<&HomeButton> {
        self.home_button.as_ref()
    }

    /// Gets InfoBox widget.
    pub fn info_box(&self) -> Option<&InfoBox> {
        self.info_box.as_ref()
    }

    /// Gets SceneModePicker widget.
    pub fn scene_mode_picker(&self) -> Option<&SceneModePicker> {
        self.scene_mode_picker.as_ref()
    }

    /// Gets SelectionIndicator widget.
    pub fn selection_indicator(&self) -> Option<&SelectionIndicator> {
        self.selection_indicator.as_ref()
    }

    /// Gets Timeline widget.
    pub fn timeline(&self) -> Option<&Timeline> {
        self.timeline.as_ref()
    }

    /// Gets NavigationHelpButton widget.
    pub fn navigation_help_button(&self) -> Option<&NavigationHelpButton> {
        self.navigation_help_button.as_ref()
    }

    /// Gets PerformanceWatchdog widget.
    pub fn performance_watchdog(&self) -> Option<&PerformanceWatchdog> {
        self.performance_watchdog.as_ref()
    }

    /// Extends Viewer with drag and drop mixin.
    pub fn extend_drag_drop(&mut self, options: Option<ViewerDragDropOptions>) {
        let opts = options.unwrap_or_default();
        self.drop_target = Some(opts.drop_target.unwrap_or_else(|| self.container.id.clone()));
        self.drop_enabled = true;
        self.clear_on_drop = opts.clear_on_drop;
        self.clamp_to_ground = opts.clamp_to_ground;
        self.fly_to_on_drop = opts.fly_to_on_drop;
    }

    /// Extends Viewer with performance watchdog mixin.
    pub fn extend_performance_watchdog(
        &mut self,
        options: Option<ViewerPerformanceWatchdogOptions>,
    ) -> Result<(), DeveloperError> {
        let opts = options.unwrap_or_default();
        let pwd_container = Rc::new(RefCell::new(PerformanceWatchdogContainer::new(format!("{}_pwd", self.container.id))));
        let pwd_scene = Rc::new(RefCell::new(WatchdogScene::default()));
        let watchdog_opts = PerformanceWatchdogOptions {
            container: Some(pwd_container),
            scene: Some(pwd_scene),
            low_frame_rate_message: opts.low_frame_rate_message,
        };
        let watchdog = PerformanceWatchdog::new(Some(watchdog_opts))?;
        self.performance_watchdog = Some(watchdog);
        Ok(())
    }

    /// Resizes viewer and all underlying child widgets.
    pub fn resize(&mut self) {
        if let Some(tl) = &mut self.timeline {
            tl.resize();
        }
    }

    /// Renders a frame.
    pub fn render(&mut self) {
        if let Some(tl) = &mut self.timeline {
            tl.update_from_clock();
        }
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the viewer and all owned widgets.
    pub fn destroy(&mut self) {
        self.is_destroyed.set(true);
        if let Some(a) = self.animation.take() {
            a.destroy();
        }
        if let Some(b) = self.base_layer_picker.take() {
            b.destroy();
        }
        if let Some(f) = self.fullscreen_button.take() {
            f.destroy();
        }
        if let Some(v) = self.vr_button.take() {
            v.destroy();
        }
        if let Some(g) = self.geocoder.take() {
            g.destroy();
        }
        if let Some(h) = self.home_button.take() {
            h.destroy();
        }
        if let Some(mut i) = self.info_box.take() {
            i.destroy();
        }
        if let Some(mut s) = self.scene_mode_picker.take() {
            s.destroy();
        }
        if let Some(mut s) = self.selection_indicator.take() {
            s.destroy();
        }
        if let Some(mut t) = self.timeline.take() {
            t.destroy();
        }
        if let Some(mut n) = self.navigation_help_button.take() {
            n.destroy();
        }
        if let Some(mut p) = self.performance_watchdog.take() {
            p.destroy();
        }
    }
}

/// Applies `viewerDragDropMixin` to viewer.
pub fn viewer_drag_drop_mixin(
    viewer: Option<&mut Viewer>,
    options: Option<ViewerDragDropOptions>,
) -> Result<(), DeveloperError> {
    let viewer = viewer.ok_or_else(|| DeveloperError::new("viewer is required."))?;
    viewer.extend_drag_drop(options);
    Ok(())
}

/// Applies `viewerPerformanceWatchdogMixin` to viewer.
pub fn viewer_performance_watchdog_mixin(
    viewer: Option<&mut Viewer>,
    options: Option<ViewerPerformanceWatchdogOptions>,
) -> Result<(), DeveloperError> {
    let viewer = viewer.ok_or_else(|| DeveloperError::new("viewer is required."))?;
    viewer.extend_performance_watchdog(options)
}
