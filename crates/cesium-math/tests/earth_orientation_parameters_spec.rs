use std::path::PathBuf;
use cesium_math::{
    EarthOrientationParameters, EarthOrientationParametersData, JulianDate,
    TimeStandard,
};

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn linear_interp(dx: f64, y1: f64, y2: f64) -> f64 {
    y1 + dx * (y2 - y1)
}

fn test_data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("cesium/Specs/Data")
        .join(relative)
}

#[test]
fn test_adds_leap_seconds_found_in_the_data_by_default() {
    let _guard = TEST_MUTEX.lock().unwrap();
    JulianDate::reset_leap_seconds();
    let official_leap_seconds = JulianDate::leap_seconds();

    let column_names = vec![
        "dateIso8601".to_string(),
        "modifiedJulianDateUtc".to_string(),
        "xPoleWanderRadians".to_string(),
        "yPoleWanderRadians".to_string(),
        "xCelestialPoleOffsetRadians".to_string(),
        "yCelestialPoleOffsetRadians".to_string(),
        "ut1MinusUtcSeconds".to_string(),
        "taiMinusUtcSeconds".to_string(),
        "lengthOfDayCorrectionSeconds".to_string(),
    ];
    let samples: Vec<serde_json::Value> = vec![
        serde_json::json!("2011-08-25T00:00:00Z"),
        serde_json::json!(55798.0),
        serde_json::json!(2.117957047295119e-7),
        serde_json::json!(2.111518721609984e-6),
        serde_json::json!(3.393695767766752e-11),
        serde_json::json!(3.3452143996557983e-10),
        serde_json::json!(-0.2908948),
        serde_json::json!(34.0),
        serde_json::json!(-2.956e-4),
        serde_json::json!("2011-08-26T00:00:00Z"),
        serde_json::json!(55799.0),
        serde_json::json!(2.193297093339541e-7),
        serde_json::json!(2.115460256837405e-6),
        serde_json::json!(-8.241832578862112e-11),
        serde_json::json!(5.623838700870617e-10),
        serde_json::json!(-0.29065),
        serde_json::json!(34.5),
        serde_json::json!(-1.824e-4),
        serde_json::json!("2011-08-27T00:00:00Z"),
        serde_json::json!(55800.0),
        serde_json::json!(2.262286080161428e-7),
        serde_json::json!(2.1191157519929706e-6),
        serde_json::json!(-3.490658503988659e-10),
        serde_json::json!(6.981317007977318e-10),
        serde_json::json!(-0.2905572),
        serde_json::json!(34.5),
        serde_json::json!(1.9e-6),
        serde_json::json!("2011-08-28T00:00:00Z"),
        serde_json::json!(55801.0),
        serde_json::json!(2.3411652660779493e-7),
        serde_json::json!(2.122751854601292e-6),
        serde_json::json!(-6.205615118202061e-10),
        serde_json::json!(7.853981633974483e-10),
        serde_json::json!(-0.2907007),
        serde_json::json!(34.5),
        serde_json::json!(2.695e-4),
    ];

    let eop = EarthOrientationParameters::new(
        Some(EarthOrientationParametersData {
            column_names,
            samples,
        }),
        true,
    );
    assert!(eop.is_ok());

    let leap_seconds = JulianDate::leap_seconds();
    assert_eq!(leap_seconds.len(), official_leap_seconds.len() + 1);

    let new_date = JulianDate::new(2455799.5, 34.5, TimeStandard::TAI);
    let mut found_new = false;
    let mut previous_date: Option<JulianDate> = None;

    for leap_second in &leap_seconds {
        if leap_second.julian_date.equals(&new_date) {
            found_new = true;
        }
        if let Some(prev) = previous_date {
            assert!(prev.compare(&leap_second.julian_date).is_lt());
        }
        previous_date = Some(leap_second.julian_date);
    }

    assert!(found_new);
    JulianDate::reset_leap_seconds();
}

#[test]
fn test_does_not_add_leap_seconds_if_told_not_to_do_so() {
    let _guard = TEST_MUTEX.lock().unwrap();
    JulianDate::reset_leap_seconds();
    let official_leap_seconds = JulianDate::leap_seconds();

    let column_names = vec![
        "dateIso8601".to_string(),
        "modifiedJulianDateUtc".to_string(),
        "xPoleWanderRadians".to_string(),
        "yPoleWanderRadians".to_string(),
        "xCelestialPoleOffsetRadians".to_string(),
        "yCelestialPoleOffsetRadians".to_string(),
        "ut1MinusUtcSeconds".to_string(),
        "taiMinusUtcSeconds".to_string(),
        "lengthOfDayCorrectionSeconds".to_string(),
    ];
    let samples: Vec<serde_json::Value> = vec![
        serde_json::json!("2011-08-25T00:00:00Z"),
        serde_json::json!(55798.0),
        serde_json::json!(2.117957047295119e-7),
        serde_json::json!(2.111518721609984e-6),
        serde_json::json!(3.393695767766752e-11),
        serde_json::json!(3.3452143996557983e-10),
        serde_json::json!(-0.2908948),
        serde_json::json!(35.0),
        serde_json::json!(-2.956e-4),
        serde_json::json!("2011-08-26T00:00:00Z"),
        serde_json::json!(55799.0),
        serde_json::json!(2.193297093339541e-7),
        serde_json::json!(2.115460256837405e-6),
        serde_json::json!(-8.241832578862112e-11),
        serde_json::json!(5.623838700870617e-10),
        serde_json::json!(-0.29065),
        serde_json::json!(36.0),
        serde_json::json!(-1.824e-4),
        serde_json::json!("2011-08-27T00:00:00Z"),
        serde_json::json!(55800.0),
        serde_json::json!(2.262286080161428e-7),
        serde_json::json!(2.1191157519929706e-6),
        serde_json::json!(-3.490658503988659e-10),
        serde_json::json!(6.981317007977318e-10),
        serde_json::json!(-0.2905572),
        serde_json::json!(36.0),
        serde_json::json!(1.9e-6),
        serde_json::json!("2011-08-28T00:00:00Z"),
        serde_json::json!(55801.0),
        serde_json::json!(2.3411652660779493e-7),
        serde_json::json!(2.122751854601292e-6),
        serde_json::json!(-6.205615118202061e-10),
        serde_json::json!(7.853981633974483e-10),
        serde_json::json!(-0.2907007),
        serde_json::json!(36.0),
        serde_json::json!(2.695e-4),
    ];

    let eop = EarthOrientationParameters::new(
        Some(EarthOrientationParametersData {
            column_names,
            samples,
        }),
        false,
    );
    assert!(eop.is_ok());

    let leap_seconds = JulianDate::leap_seconds();
    assert_eq!(leap_seconds.len(), official_leap_seconds.len());
    JulianDate::reset_leap_seconds();
}

#[test]
fn test_interpolates_data_correctly_under_normal_circumstances() {
    let column_names = vec![
        "dateIso8601".to_string(),
        "modifiedJulianDateUtc".to_string(),
        "xPoleWanderRadians".to_string(),
        "yPoleWanderRadians".to_string(),
        "xCelestialPoleOffsetRadians".to_string(),
        "yCelestialPoleOffsetRadians".to_string(),
        "ut1MinusUtcSeconds".to_string(),
        "taiMinusUtcSeconds".to_string(),
        "lengthOfDayCorrectionSeconds".to_string(),
    ];
    let samples: Vec<serde_json::Value> = vec![
        serde_json::json!("2011-07-01T00:00:00Z"),
        serde_json::json!(55743.0),
        serde_json::json!(2.117957047295119e-7),
        serde_json::json!(2.111518721609984e-6),
        serde_json::json!(3.393695767766752e-11),
        serde_json::json!(3.3452143996557983e-10),
        serde_json::json!(-0.2908948),
        serde_json::json!(34.0),
        serde_json::json!(-2.956e-4),
        serde_json::json!("2011-07-02T00:00:00Z"),
        serde_json::json!(55744.0),
        serde_json::json!(2.193297093339541e-7),
        serde_json::json!(2.115460256837405e-6),
        serde_json::json!(-8.241832578862112e-11),
        serde_json::json!(5.623838700870617e-10),
        serde_json::json!(-0.29065),
        serde_json::json!(34.0),
        serde_json::json!(-1.824e-4),
        serde_json::json!("2011-07-03T00:00:00Z"),
        serde_json::json!(55745.0),
        serde_json::json!(2.262286080161428e-7),
        serde_json::json!(2.1191157519929706e-6),
        serde_json::json!(-3.490658503988659e-10),
        serde_json::json!(6.981317007977318e-10),
        serde_json::json!(-0.2905572),
        serde_json::json!(34.0),
        serde_json::json!(1.9e-6),
        serde_json::json!("2011-07-04T00:00:00Z"),
        serde_json::json!(55746.0),
        serde_json::json!(2.3411652660779493e-7),
        serde_json::json!(2.122751854601292e-6),
        serde_json::json!(-6.205615118202061e-10),
        serde_json::json!(7.853981633974483e-10),
        serde_json::json!(-0.2907007),
        serde_json::json!(34.0),
        serde_json::json!(2.695e-4),
    ];

    let mut eop = EarthOrientationParameters::new(
        Some(EarthOrientationParametersData {
            column_names: column_names.clone(),
            samples: samples.clone(),
        }),
        true,
    )
    .unwrap();

    let date = JulianDate::from_iso8601("2011-07-02T12:34:56Z").unwrap();
    let result = eop.compute(date);

    let n_columns = column_names.len();
    let sample_f64 = |idx: usize| -> f64 { samples[idx].as_f64().unwrap() };

    let mut x0 = sample_f64(n_columns + 2);
    let mut x1 = sample_f64(2 * n_columns + 2);
    let t0 = JulianDate::from_iso8601(samples[n_columns].as_str().unwrap()).unwrap();
    let dt = date.seconds_difference(&t0) / 86400.0;

    let mut expected = linear_interp(dt, x0, x1);
    assert!((result.x_pole_wander - expected).abs() < 1e-12);

    x0 = sample_f64(n_columns + 3);
    x1 = sample_f64(2 * n_columns + 3);
    expected = linear_interp(dt, x0, x1);
    assert!((result.y_pole_wander - expected).abs() < 1e-12);

    x0 = sample_f64(n_columns + 4);
    x1 = sample_f64(2 * n_columns + 4);
    expected = linear_interp(dt, x0, x1);
    assert!((result.x_pole_offset - expected).abs() < 1e-12);

    x0 = sample_f64(n_columns + 5);
    x1 = sample_f64(2 * n_columns + 5);
    expected = linear_interp(dt, x0, x1);
    assert!((result.y_pole_offset - expected).abs() < 1e-12);

    x0 = sample_f64(n_columns + 6);
    x1 = sample_f64(2 * n_columns + 6);
    expected = linear_interp(dt, x0, x1);
    assert!((result.ut1_minus_utc - expected).abs() < 1e-12);
}

#[test]
fn test_interpolates_ut1_correctly_over_a_leap_second() {
    let column_names = vec![
        "dateIso8601".to_string(),
        "modifiedJulianDateUtc".to_string(),
        "xPoleWanderRadians".to_string(),
        "yPoleWanderRadians".to_string(),
        "xCelestialPoleOffsetRadians".to_string(),
        "yCelestialPoleOffsetRadians".to_string(),
        "ut1MinusUtcSeconds".to_string(),
        "taiMinusUtcSeconds".to_string(),
        "lengthOfDayCorrectionSeconds".to_string(),
    ];
    let samples: Vec<serde_json::Value> = vec![
        serde_json::json!("2008-12-30T00:00:00Z"),
        serde_json::json!(54830.0),
        serde_json::json!(-4.6702101901281595e-8),
        serde_json::json!(6.989752766028624e-7),
        serde_json::json!(-7.80550026586353e-10),
        serde_json::json!(-1.052045688007693e-9),
        serde_json::json!(-0.5911461),
        serde_json::json!(33.0),
        serde_json::json!(5.7e-4),
        serde_json::json!("2008-12-31T00:00:00Z"),
        serde_json::json!(54831.0),
        serde_json::json!(-6.507169227852191e-8),
        serde_json::json!(7.02873178598983e-7),
        serde_json::json!(-7.80550026586353e-10),
        serde_json::json!(-1.8907733563271904e-9),
        serde_json::json!(-0.59187),
        serde_json::json!(33.0),
        serde_json::json!(8.303e-4),
        serde_json::json!("2009-01-01T00:00:00Z"),
        serde_json::json!(54832.0),
        serde_json::json!(-8.25443773457096e-8),
        serde_json::json!(7.089575902969077e-7),
        serde_json::json!(-8.047907106418297e-10),
        serde_json::json!(-2.0604581447155277e-9),
        serde_json::json!(0.4071533),
        serde_json::json!(34.0),
        serde_json::json!(0.0010604),
        serde_json::json!("2009-01-02T00:00:00Z"),
        serde_json::json!(54833.0),
        serde_json::json!(-9.884381330461219e-8),
        serde_json::json!(7.147559619229779e-7),
        serde_json::json!(-8.435758051305926e-10),
        serde_json::json!(-1.8228994409718553e-9),
        serde_json::json!(0.4059739),
        serde_json::json!(34.0),
        serde_json::json!(0.0012279),
        serde_json::json!("2009-01-03T00:00:00Z"),
        serde_json::json!(54834.0),
        serde_json::json!(-1.1269009203710055e-7),
        serde_json::json!(7.226826656091187e-7),
        serde_json::json!(-9.066015836748322e-10),
        serde_json::json!(-1.5029224114395617e-9),
        serde_json::json!(0.404674),
        serde_json::json!(34.0),
        serde_json::json!(0.0013014),
    ];

    let mut eop = EarthOrientationParameters::new(
        Some(EarthOrientationParametersData {
            column_names: column_names.clone(),
            samples: samples.clone(),
        }),
        true,
    )
    .unwrap();

    let sample_f64 = |idx: usize| -> f64 { samples[idx].as_f64().unwrap() };
    let date_at_leap_second = JulianDate::from_iso8601("2009-01-01T00:00:00Z").unwrap();
    let date_slightly_before = date_at_leap_second.add_seconds(-1.0);
    let date_slightly_after = date_at_leap_second.add_seconds(1.0);

    let n_columns = column_names.len();
    let x0 = sample_f64(n_columns + 6);
    let x1 = sample_f64(2 * n_columns + 6);
    let x2 = sample_f64(3 * n_columns + 6);

    let t0 = JulianDate::from_iso8601(samples[n_columns].as_str().unwrap()).unwrap();
    let t1 = JulianDate::from_iso8601(samples[2 * n_columns].as_str().unwrap()).unwrap();

    let mut dt = date_slightly_before.seconds_difference(&t0) / (86400.0 + 1.0);
    let expected_before = linear_interp(dt, x0, x1 - 1.0);
    let result_before = eop.compute(date_slightly_before);
    assert!((result_before.ut1_minus_utc - expected_before).abs() < 1e-12);

    let expected_at = sample_f64(2 * n_columns + 6);
    let result_at = eop.compute(date_at_leap_second);
    assert!((result_at.ut1_minus_utc - expected_at).abs() < 1e-12);

    dt = date_slightly_after.seconds_difference(&t1) / 86400.0;
    let expected_after = linear_interp(dt, x1, x2);
    let result_after = eop.compute(date_slightly_after);
    assert!((result_after.ut1_minus_utc - expected_after).abs() < 1e-12);

    // Discontinuous check
    assert!((result_before.ut1_minus_utc - result_after.ut1_minus_utc).abs() > 0.5);
}

#[test]
fn test_from_file_loads_eop_data() {
    let path = test_data_path("EarthOrientationParameters/EOP-2011-July.json");
    let mut eop = EarthOrientationParameters::from_file(path, true).expect("loads valid EOP");

    // 2011-07-03 00:00:00 UTC
    let time = JulianDate::new(2455745.0, 43200.0, TimeStandard::UTC);
    let result = eop.compute(time);
    assert!(result.x_pole_wander != 0.0);
    assert!(result.y_pole_wander != 0.0);
    assert!(result.x_pole_offset != 0.0);
    assert!(result.y_pole_offset != 0.0);
    assert!(result.ut1_minus_utc != 0.0);
}

#[test]
fn test_from_file_throws_error_when_loading_invalid_data() {
    let path = test_data_path("EarthOrientationParameters/EOP-Invalid.json");
    let err = EarthOrientationParameters::from_file(path, true).unwrap_err();
    assert!(err.message.contains("The columnNames property is required."));
}

#[test]
fn test_from_file_throws_error_when_missing_file() {
    let path = test_data_path("EarthOrientationParameters/EOP-DoesNotExist.json");
    let err = EarthOrientationParameters::from_file(&path, true).unwrap_err();
    assert!(err.message.contains("An error occurred while retrieving the EOP data from the URL"));
}
