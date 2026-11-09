use cesium_math::DeveloperError;

/// Built-in job categories for frame time slicing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum JobType {
    Texture = 0,
    Program = 1,
    Buffer = 2,
}

impl JobType {
    pub const NUMBER_OF_JOB_TYPES: usize = 3;

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Texture),
            1 => Some(Self::Program),
            2 => Some(Self::Buffer),
            _ => None,
        }
    }
}

/// Budget allocation and tracking for a single job category.
#[derive(Debug, Clone, PartialEq)]
pub struct JobTypeBudget {
    pub total: f64,
    pub used_this_frame: f64,
    pub stolen_from_me_this_frame: f64,
    pub starved_this_frame: bool,
    pub starved_last_frame: bool,
}

impl JobTypeBudget {
    pub fn new(total: f64) -> Self {
        Self {
            total,
            used_this_frame: 0.0,
            stolen_from_me_this_frame: 0.0,
            starved_this_frame: false,
            starved_last_frame: false,
        }
    }
}

pub trait ExecutableJob {
    fn execute(&mut self);
}

impl<F: FnMut()> ExecutableJob for F {
    fn execute(&mut self) {
        self();
    }
}

/// Time-slicing engine to amortize jobs across frames while guaranteeing forward progress.
pub struct JobScheduler {
    pub total_budget: f64,
    pub total_used_this_frame: f64,
    pub budgets: [JobTypeBudget; JobType::NUMBER_OF_JOB_TYPES],
    pub executed_this_frame: [bool; JobType::NUMBER_OF_JOB_TYPES],
    timestamp_fn: Option<Box<dyn FnMut() -> f64 + Send>>,
}

impl std::fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobScheduler")
            .field("total_budget", &self.total_budget)
            .field("total_used_this_frame", &self.total_used_this_frame)
            .field("budgets", &self.budgets)
            .field("executed_this_frame", &self.executed_this_frame)
            .finish()
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new(None).expect("valid default budgets")
    }
}

impl JobScheduler {
    /// Creates a scheduler with custom or default budgets for [TEXTURE, PROGRAM, BUFFER].
    pub fn new(budgets: Option<&[f64]>) -> Result<Self, DeveloperError> {
        if let Some(b) = budgets {
            if b.len() != JobType::NUMBER_OF_JOB_TYPES {
                return Err(DeveloperError::new(
                    "A budget must be specified for each job type; budgets.length should equal JobType.NUMBER_OF_JOB_TYPES.",
                ));
            }
        }

        let texture_budget = budgets.map(|b| b[0]).unwrap_or(10.0);
        let program_budget = budgets.map(|b| b[1]).unwrap_or(10.0);
        let buffer_budget = budgets.map(|b| b[2]).unwrap_or(30.0);

        let job_budgets = [
            JobTypeBudget::new(texture_budget),
            JobTypeBudget::new(program_budget),
            JobTypeBudget::new(buffer_budget),
        ];

        let total_budget = texture_budget + program_budget + buffer_budget;

        Ok(Self {
            total_budget,
            total_used_this_frame: 0.0,
            budgets: job_budgets,
            executed_this_frame: [false; JobType::NUMBER_OF_JOB_TYPES],
            timestamp_fn: None,
        })
    }

    /// Sets a custom timestamp provider (milliseconds) for deterministic simulation/testing.
    pub fn set_timestamp_provider<F: FnMut() -> f64 + Send + 'static>(&mut self, f: F) {
        self.timestamp_fn = Some(Box::new(f));
    }

    fn now(&mut self) -> f64 {
        if let Some(ref mut ts) = self.timestamp_fn {
            ts()
        } else {
            use std::time::SystemTime;
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0)
        }
    }

    /// Disables scheduling for this frame by consuming the total budget.
    pub fn disable_this_frame(&mut self) {
        self.total_used_this_frame = self.total_budget;
    }

    /// Resets budgets and starvation status for a new frame.
    pub fn reset_budgets(&mut self) {
        for budget in &mut self.budgets {
            budget.starved_last_frame = budget.starved_this_frame;
            budget.starved_this_frame = false;
            budget.used_this_frame = 0.0;
            budget.stolen_from_me_this_frame = 0.0;
        }
        self.total_used_this_frame = 0.0;
        self.executed_this_frame = [false; JobType::NUMBER_OF_JOB_TYPES];
    }

    /// Attempts to execute a job under the specified job type's budget.
    pub fn execute<J: ExecutableJob>(&mut self, mut job: J, job_type: JobType) -> bool {
        let type_idx = job_type as usize;
        let progress_this_frame = self.executed_this_frame[type_idx];

        if self.total_used_this_frame >= self.total_budget && progress_this_frame {
            self.budgets[type_idx].starved_this_frame = true;
            return false;
        }

        let mut stolen_idx = None;

        if self.budgets[type_idx].used_this_frame + self.budgets[type_idx].stolen_from_me_this_frame
            >= self.budgets[type_idx].total
        {
            // Try to steal from other job types
            for i in 0..JobType::NUMBER_OF_JOB_TYPES {
                if self.budgets[i].used_this_frame + self.budgets[i].stolen_from_me_this_frame
                    < self.budgets[i].total
                    && !self.budgets[i].starved_last_frame
                {
                    stolen_idx = Some(i);
                    break;
                }
            }

            if stolen_idx.is_none() && progress_this_frame {
                self.budgets[type_idx].starved_this_frame = true;
                return false;
            }

            if progress_this_frame {
                self.budgets[type_idx].starved_this_frame = true;
            }
        }

        let start_time = self.now();
        job.execute();
        let end_time = self.now();
        let duration = end_time - start_time;

        self.total_used_this_frame += duration;

        if let Some(idx) = stolen_idx {
            self.budgets[idx].stolen_from_me_this_frame += duration;
        } else {
            self.budgets[type_idx].used_this_frame += duration;
        }
        self.executed_this_frame[type_idx] = true;

        true
    }
}
