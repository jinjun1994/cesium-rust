use std::path::PathBuf;
use std::time::{Duration, Instant};
use cesium_math::{Iau2006XysData, Iau2006XysDataOptions, Iau2006XysSample};

fn poll_to_some<T, F: FnMut() -> Option<T>>(timeout: Duration, mut f: F) -> Option<T> {
    let start = Instant::now();
    loop {
        if let Some(res) = f() {
            return Some(res);
        }
        if start.elapsed() > timeout {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn asset_template_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("cesium/packages/engine/Source/Assets/IAU2006_XYS/IAU2006_XYS_{0}.json")
        .to_str()
        .unwrap()
        .to_string()
}

#[test]
fn test_returns_none_initially() {
    let mut xys = Iau2006XysData::default();
    assert!(xys.compute_xys_radians(2442398, 1234.56).is_none());
}

#[test]
fn test_eventually_returns_an_answer() {
    let mut xys = Iau2006XysData::default();
    let res = poll_to_some(Duration::from_secs(3), || {
        xys.compute_xys_radians(2442398, 1234.56)
    });
    assert!(res.is_some(), "expected XYS computation to succeed after background load");

    // Once the data file has been downloaded, later requests within the same chunk return an answer.
    let second_res = xys.compute_xys_radians(2442399, 777.77);
    assert!(second_res.is_some());
}

#[test]
#[allow(clippy::excessive_precision)]
fn test_returns_the_same_answer_as_stk_components() {
    let mut xys = Iau2006XysData::default();
    let result = poll_to_some(Duration::from_secs(3), || {
        xys.compute_xys_radians(2442399, 777.77)
    })
    .expect("expected result within timeout");

    let expected = Iau2006XysSample::new(
        -0.0024019733101066816,
        -0.000024843279494458311,
        -0.000000016941747917421229,
    );

    assert!(
        result.equals_epsilon(&expected, 1e-15),
        "Result {:?} did not match expected {:?}",
        result,
        expected
    );
}

#[test]
fn test_returns_none_prior_to_the_xys_table_epoch() {
    let mut xys = Iau2006XysData::default();
    assert!(xys.compute_xys_radians(2442395, 0.0).is_none());
}

#[test]
fn test_returns_none_after_the_last_xys_table_sample() {
    let mut xys = Iau2006XysData::default();
    assert!(xys.compute_xys_radians(2442396 + 27427, 0.0).is_none());
}

#[test]
fn test_allows_configuring_xys_file_url_template() {
    let template = asset_template_path();
    let mut xys = Iau2006XysData::new(Some(Iau2006XysDataOptions {
        xys_file_url_template: Some(template),
        ..Default::default()
    }));

    let res = poll_to_some(Duration::from_secs(3), || {
        xys.compute_xys_radians(2442398, 1234.56)
    });
    assert!(res.is_some(), "expected configured template to load successfully");
}

#[test]
fn test_preload_and_synchronous_load() {
    let mut xys = Iau2006XysData::default();
    xys.preload(2442398, 0.0, 2442400, 0.0);

    let res = poll_to_some(Duration::from_secs(3), || {
        xys.compute_xys_radians(2442398, 100.0)
    });
    assert!(res.is_some());
}
