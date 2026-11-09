use cesium_renderer::job_scheduler::{ExecutableJob, JobScheduler, JobType};

#[derive(Default)]
struct MockJob {
    executed: bool,
}

impl ExecutableJob for &mut MockJob {
    fn execute(&mut self) {
        self.executed = true;
    }
}

fn create_deterministic_scheduler(budgets: Option<&[f64]>) -> JobScheduler {
    let mut js = JobScheduler::new(budgets).unwrap();
    let mut current_time = 0.0f64;
    js.set_timestamp_provider(move || {
        let t = current_time;
        current_time += 1.0;
        t
    });
    js
}

#[test]
fn constructs_with_defaults() {
    let js = JobScheduler::new(None).unwrap();
    assert_eq!(js.total_budget, 50.0);
    assert_eq!(js.budgets[JobType::Texture as usize].total, 10.0);
    assert_eq!(js.budgets[JobType::Program as usize].total, 10.0);
    assert_eq!(js.budgets[JobType::Buffer as usize].total, 30.0);
}

#[test]
fn constructor_throws_when_budgets_length_invalid() {
    assert!(JobScheduler::new(Some(&[1.0])).is_err());
    assert!(JobScheduler::new(Some(&[1.0, 2.0])).is_err());
    assert!(JobScheduler::new(Some(&[1.0, 2.0, 3.0, 4.0])).is_err());
}

#[test]
fn executes_a_job() {
    let mut js = create_deterministic_scheduler(Some(&[2.0, 0.0, 0.0]));
    let mut job = MockJob::default();
    let executed = js.execute(&mut job, JobType::Texture);

    assert!(executed);
    assert!(job.executed);
    assert_eq!(js.total_used_this_frame, 1.0);
    assert_eq!(js.budgets[JobType::Texture as usize].total, 2.0);
    assert_eq!(js.budgets[JobType::Texture as usize].used_this_frame, 1.0);
}

#[test]
fn disable_this_frame_does_not_execute_job() {
    let mut js = create_deterministic_scheduler(Some(&[2.0, 0.0, 0.0]));
    let mut job1 = MockJob::default();
    assert!(js.execute(&mut job1, JobType::Texture));

    js.disable_this_frame();
    let mut job2 = MockJob::default();
    assert!(!js.execute(&mut job2, JobType::Texture));
}

#[test]
fn executes_different_job_types() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 1.0, 1.0]));
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Program));
    assert!(js.execute(|| {}, JobType::Buffer));

    assert_eq!(js.total_used_this_frame, 3.0);
    assert_eq!(js.budgets[JobType::Texture as usize].used_this_frame, 1.0);
    assert_eq!(js.budgets[JobType::Program as usize].used_this_frame, 1.0);
    assert_eq!(js.budgets[JobType::Buffer as usize].used_this_frame, 1.0);
}

#[test]
fn executes_a_second_job() {
    let mut js = create_deterministic_scheduler(Some(&[2.0, 0.0, 0.0]));
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Texture));
    assert_eq!(js.total_used_this_frame, 2.0);
    assert_eq!(js.budgets[JobType::Texture as usize].used_this_frame, 2.0);
}

#[test]
fn does_not_execute_second_job_when_exceeds_total_time() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 0.0, 0.0]));
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(!js.execute(|| {}, JobType::Texture));
    assert!(js.budgets[JobType::Texture as usize].starved_this_frame);
}

#[test]
fn executes_a_second_job_texture_steals_program_budget() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 1.0, 0.0]));
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Texture));
    assert_eq!(js.total_used_this_frame, 2.0);

    let b_tex = &js.budgets[JobType::Texture as usize];
    assert_eq!(b_tex.used_this_frame, 1.0);
    assert!(b_tex.starved_this_frame);

    let b_prog = &js.budgets[JobType::Program as usize];
    assert_eq!(b_prog.used_this_frame, 0.0);
    assert_eq!(b_prog.stolen_from_me_this_frame, 1.0);
    assert!(!b_prog.starved_this_frame);

    // There are no budgets left to steal from
    assert!(!js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Program)); // Allowed once per frame
    assert!(!js.execute(|| {}, JobType::Program));
    assert!(js.budgets[JobType::Program as usize].starved_this_frame);
}

#[test]
fn does_not_steal_in_the_same_frame() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 1.0, 1.0]));
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Program));
    assert!(js.execute(|| {}, JobType::Buffer));

    // Exhaust budget for all job types in the first frame
    assert!(!js.execute(|| {}, JobType::Texture));
    assert!(!js.execute(|| {}, JobType::Program));
    assert!(!js.execute(|| {}, JobType::Buffer));

    // In this next frame, no job type can steal from another since
    // they all exhausted their budgets in the previous frame
    js.reset_budgets();
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(!js.execute(|| {}, JobType::Texture));

    assert!(js.execute(|| {}, JobType::Program));
    assert!(!js.execute(|| {}, JobType::Program));

    assert!(js.execute(|| {}, JobType::Buffer));
    assert!(!js.execute(|| {}, JobType::Buffer));
}

#[test]
fn does_not_steal_from_starving_job_types_over_multiple_frames() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 1.0, 0.0]));

    // Exhaust in first frame
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Texture)); // Stolen from PROGRAM
    assert!(!js.execute(|| {}, JobType::Texture));

    js.reset_budgets();
    assert!(js.execute(|| {}, JobType::Program));
    assert!(!js.execute(|| {}, JobType::Program)); // Can't steal from TEXTURE
    assert!(js.execute(|| {}, JobType::Texture));
    assert!(!js.execute(|| {}, JobType::Texture));

    js.reset_budgets();
    assert!(js.execute(|| {}, JobType::Program));
    assert!(!js.execute(|| {}, JobType::Program)); // Can't steal from TEXTURE yet

    js.reset_budgets();
    assert!(js.execute(|| {}, JobType::Program));
    assert!(js.execute(|| {}, JobType::Program)); // Can steal from TEXTURE since it wasn't exhausted last frame
}

#[test]
fn allows_progress_on_all_job_types_once_per_frame() {
    let mut js = create_deterministic_scheduler(Some(&[1.0, 1.0, 1.0]));

    assert!(js.execute(|| {}, JobType::Texture));
    assert!(js.execute(|| {}, JobType::Texture)); // Steal from PROGRAM
    assert!(js.execute(|| {}, JobType::Texture)); // Steal from BUFFER
    assert!(!js.execute(|| {}, JobType::Texture));

    assert!(js.execute(|| {}, JobType::Program)); // Still gets to make progress once this frame
    assert!(!js.execute(|| {}, JobType::Program));

    assert!(js.execute(|| {}, JobType::Buffer)); // Still gets to make progress once this frame
    assert!(!js.execute(|| {}, JobType::Buffer));
}

#[test]
fn long_job_still_allows_progress_on_other_job_types_once_per_frame() {
    let mut js = create_deterministic_scheduler(Some(&[0.5, 0.2, 0.2]));
    assert!(js.execute(|| {}, JobType::Texture)); // Goes over total budget
    assert!(js.execute(|| {}, JobType::Program)); // Still gets to make progress once this frame
    assert!(js.execute(|| {}, JobType::Buffer)); // Still gets to make progress once this frame
}
