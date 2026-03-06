use std::ffi::c_void;

use hyprs_sdk::plugin::*;

// ── LayoutHandle (legacy) ──

#[test]
fn layout_handle_null() {
    let h = LayoutHandle::NULL;
    assert!(h.is_null());
}

#[test]
fn layout_handle_non_null() {
    let mut dummy: u8 = 0;
    let h = LayoutHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!h.is_null());
}

// ── Direction ──

#[test]
fn direction_default() {
    let d = Direction::default();
    assert_eq!(d, Direction::Default);
}

#[test]
fn direction_variants() {
    assert_eq!(Direction::Up as i8, 0);
    assert_eq!(Direction::Right as i8, 1);
    assert_eq!(Direction::Down as i8, 2);
    assert_eq!(Direction::Left as i8, 3);
}

// ── RectCorner ──

#[test]
fn rect_corner_default() {
    let c = RectCorner::default();
    assert_eq!(c, RectCorner::None);
}

#[test]
fn rect_corner_values() {
    assert_eq!(RectCorner::TopLeft as u8, 1);
    assert_eq!(RectCorner::TopRight as u8, 2);
    assert_eq!(RectCorner::BottomRight as u8, 4);
    assert_eq!(RectCorner::BottomLeft as u8, 8);
}

// ── FocalPoint ──

#[test]
fn focal_point_fields() {
    let fp = FocalPoint { x: 1.5, y: 2.5 };
    assert!((fp.x - 1.5).abs() < f64::EPSILON);
    assert!((fp.y - 2.5).abs() < f64::EPSILON);
}

#[test]
fn focal_point_clone() {
    let fp = FocalPoint { x: 3.0, y: 4.0 };
    let fp2 = fp;
    assert!((fp2.x - 3.0).abs() < f64::EPSILON);
    assert!((fp2.y - 4.0).abs() < f64::EPSILON);
}

// ── Legacy Layout trait ──

#[test]
fn layout_trait_object_safety() {
    fn _assert_object_safe(_: &dyn Layout) {}
}

// ── ModeAlgorithm trait ──

struct DummyTiled;

impl ModeAlgorithm for DummyTiled {
    fn new_target(&mut self, _: *mut c_void) {}
    fn moved_target(&mut self, _: *mut c_void, _: Option<FocalPoint>) {}
    fn remove_target(&mut self, _: *mut c_void) {}
    fn resize_target(&mut self, _: f64, _: f64, _: *mut c_void, _: RectCorner) {}
    fn recalculate(&mut self) {}
    fn swap_targets(&mut self, _: *mut c_void, _: *mut c_void) {}
    fn move_target_in_direction(&mut self, _: *mut c_void, _: Direction, _: bool) {}
}

impl TiledAlgorithm for DummyTiled {}

#[test]
fn tiled_algorithm_default_candidate_is_null() {
    let mut algo = DummyTiled;
    let result = algo.get_next_candidate(std::ptr::null_mut());
    assert!(result.is_null());
}

#[test]
fn mode_algorithm_default_layout_msg_returns_err() {
    let mut algo = DummyTiled;
    let result = algo.layout_msg("anything");
    assert!(result.is_err());
}

#[test]
fn mode_algorithm_default_predict_size_returns_none() {
    let algo = DummyTiled;
    assert!(algo.predict_size_for_new_target().is_none());
}

// ── FloatingAlgorithm trait ──

struct DummyFloating {
    last_geom: Option<(f64, f64, f64, f64)>,
    last_delta: Option<(f64, f64)>,
}

impl ModeAlgorithm for DummyFloating {
    fn new_target(&mut self, _: *mut c_void) {}
    fn moved_target(&mut self, _: *mut c_void, _: Option<FocalPoint>) {}
    fn remove_target(&mut self, _: *mut c_void) {}
    fn resize_target(&mut self, _: f64, _: f64, _: *mut c_void, _: RectCorner) {}
    fn recalculate(&mut self) {}
    fn swap_targets(&mut self, _: *mut c_void, _: *mut c_void) {}
    fn move_target_in_direction(&mut self, _: *mut c_void, _: Direction, _: bool) {}
}

impl FloatingAlgorithm for DummyFloating {
    fn move_target(&mut self, dx: f64, dy: f64, _: *mut c_void) {
        self.last_delta = Some((dx, dy));
    }
    fn set_target_geom(&mut self, x: f64, y: f64, w: f64, h: f64, _: *mut c_void) {
        self.last_geom = Some((x, y, w, h));
    }
}

#[test]
fn floating_algorithm_move_target() {
    let mut algo = DummyFloating {
        last_geom: None,
        last_delta: None,
    };
    algo.move_target(10.0, 20.0, std::ptr::null_mut());
    assert_eq!(algo.last_delta, Some((10.0, 20.0)));
}

#[test]
fn floating_algorithm_set_target_geom() {
    let mut algo = DummyFloating {
        last_geom: None,
        last_delta: None,
    };
    algo.set_target_geom(100.0, 200.0, 800.0, 600.0, std::ptr::null_mut());
    assert_eq!(algo.last_geom, Some((100.0, 200.0, 800.0, 600.0)));
}

// ── Factory traits ──

struct DummyTiledFactory;

impl TiledAlgorithmFactory for DummyTiledFactory {
    type Algo = DummyTiled;
    fn create(&self) -> DummyTiled {
        DummyTiled
    }
}

struct DummyFloatingFactory;

impl FloatingAlgorithmFactory for DummyFloatingFactory {
    type Algo = DummyFloating;
    fn create(&self) -> DummyFloating {
        DummyFloating {
            last_geom: None,
            last_delta: None,
        }
    }
}

#[test]
fn tiled_factory_creates_instance() {
    let factory = DummyTiledFactory;
    let mut algo = factory.create();
    assert!(algo.get_next_candidate(std::ptr::null_mut()).is_null());
}

#[test]
fn floating_factory_creates_instance() {
    let factory = DummyFloatingFactory;
    let mut algo = factory.create();
    algo.move_target(1.0, 2.0, std::ptr::null_mut());
    assert_eq!(algo.last_delta, Some((1.0, 2.0)));
}

// FFI registration needs the C++ bridge symbols — only available with plugin-ffi.

#[cfg(feature = "plugin-ffi")]
#[test]
fn register_tiled_algo_null_handle() {
    let factory = DummyTiledFactory;
    let result = register_tiled_algo(PluginHandle::NULL, "test", factory);
    assert!(result.is_err());
}

#[cfg(feature = "plugin-ffi")]
#[test]
fn register_floating_algo_null_handle() {
    let factory = DummyFloatingFactory;
    let result = register_floating_algo(PluginHandle::NULL, "test", factory);
    assert!(result.is_err());
}
