use cesium_math::{Cartesian3, TridiagonalSystemSolver};

#[test]
fn test_tridiagonal_solver_throws_invalid_lengths() {
    assert!(TridiagonalSystemSolver::solve(&[], &[], &[], &[Cartesian3::ZERO]).is_err());
    assert!(TridiagonalSystemSolver::solve(&[1.0], &[1.0], &[], &[Cartesian3::ZERO]).is_err());
    assert!(TridiagonalSystemSolver::solve(&[1.0], &[1.0], &[1.0], &[Cartesian3::ZERO]).is_err());
}

#[test]
fn test_tridiagonal_solver_three_unknowns() {
    let l = [1.0, 1.0];
    let d = [-2.175, -2.15, -2.125];
    let u = [1.0, 1.0];
    let r = [
        Cartesian3::new(-1.625, -1.625, -1.625),
        Cartesian3::new(0.5, 0.5, 0.5),
        Cartesian3::new(1.625, 1.625, 1.625),
    ];

    let actual = TridiagonalSystemSolver::solve(&l, &d, &u, &r).unwrap();
    assert_eq!(actual.len(), 3);

    let expected0 = Cartesian3::new(0.552, 0.552, 0.552);
    let expected1 = Cartesian3::new(-0.4244, -0.4244, -0.4244);
    let expected2 = Cartesian3::new(-0.9644, -0.9644, -0.9644);

    let eps = 1e-4;
    assert!((actual[0].x - expected0.x).abs() < eps);
    assert!((actual[1].x - expected1.x).abs() < eps);
    assert!((actual[2].x - expected2.x).abs() < eps);
}

#[test]
fn test_tridiagonal_solver_nine_unknowns() {
    let l = [1.0; 8];
    let d = [
        -2.0304, -2.0288, -2.0272, -2.0256, -2.024, -2.0224, -2.0208, -2.0192, -2.0176,
    ];
    let u = [1.0; 8];
    let r_scalars = [
        -1.952, 0.056, 0.064, 0.072, 0.08, 0.088, 0.096, 0.104, 1.112,
    ];
    let r: Vec<Cartesian3> = r_scalars.iter().map(|&s| Cartesian3::new(s, s, s)).collect();

    let actual = TridiagonalSystemSolver::solve(&l, &d, &u, &r).unwrap();
    assert_eq!(actual.len(), 9);

    let expected_scalars = [
        1.3513, 0.7918, 0.311, -0.0974, -0.4362, -0.7055, -0.9025, -1.0224, -1.0579,
    ];

    let eps = 1e-4;
    for i in 0..9 {
        assert!(
            (actual[i].x - expected_scalars[i]).abs() < eps,
            "Failed at index {}: got {}, expected {}",
            i,
            actual[i].x,
            expected_scalars[i]
        );
    }
}
