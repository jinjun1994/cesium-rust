use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{DeveloperError, Event};

/// Event argument passed to `before_execute` subscribers.
#[derive(Debug, Clone)]
pub struct CommandBeforeExecute<Args> {
    pub args: Args,
    pub cancel: Rc<Cell<bool>>,
}

impl<Args> CommandBeforeExecute<Args> {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            cancel: Rc::new(Cell::new(false)),
        }
    }
}

/// A command encapsulates an execution function with observable `can_execute` status
/// and pre/post execution lifecycle events.
#[derive(Clone)]
pub struct Command<Args: Clone + 'static = (), R: Clone + 'static = ()> {
    func: Rc<dyn Fn(Args) -> R>,
    can_execute: Rc<Cell<bool>>,
    pub before_execute: Event<CommandBeforeExecute<Args>>,
    pub after_execute: Event<R>,
}

impl<Args: Clone + 'static, R: Clone + 'static> std::fmt::Debug for Command<Args, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("can_execute", &self.can_execute())
            .finish()
    }
}

impl<Args: Clone + 'static, R: Clone + 'static> Command<Args, R> {
    /// Creates a command with default `can_execute = true`.
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(Args) -> R + 'static,
    {
        Self::new_with_can_execute(func, true)
    }

    /// Creates a command with explicit initial `can_execute` status.
    pub fn new_with_can_execute<F>(func: F, can_execute: bool) -> Self
    where
        F: Fn(Args) -> R + 'static,
    {
        Self {
            func: Rc::new(func),
            can_execute: Rc::new(Cell::new(can_execute)),
            before_execute: Event::new(),
            after_execute: Event::new(),
        }
    }

    /// Returns whether the command can currently be executed.
    #[inline]
    pub fn can_execute(&self) -> bool {
        self.can_execute.get()
    }

    /// Updates whether the command can currently be executed.
    #[inline]
    pub fn set_can_execute(&self, value: bool) {
        self.can_execute.set(value);
    }

    /// Executes the command with arguments.
    /// Returns `Err(DeveloperError)` if `can_execute` is false.
    /// Returns `Ok(None)` if a before_execute subscriber cancelled execution.
    /// Otherwise returns `Ok(Some(result))`.
    pub fn execute(&self, args: Args) -> Result<Option<R>, DeveloperError> {
        if !self.can_execute() {
            return Err(DeveloperError::new(
                "Cannot execute command, canExecute is false.",
            ));
        }

        let info = CommandBeforeExecute::new(args.clone());
        self.before_execute.raise_event(&info);

        if info.cancel.get() {
            return Ok(None);
        }

        let result = (self.func)(args);
        self.after_execute.raise_event(&result);
        Ok(Some(result))
    }
}

impl<R: Clone + 'static> Command<(), R> {
    /// Helper to execute parameterless commands.
    #[inline]
    pub fn call(&self) -> Result<Option<R>, DeveloperError> {
        self.execute(())
    }
}

/// Helper function mirroring CesiumJS `createCommand(func, canExecute)`.
pub fn create_command<Args: Clone + 'static, R: Clone + 'static, F>(func: F) -> Command<Args, R>
where
    F: Fn(Args) -> R + 'static,
{
    Command::new(func)
}

/// Helper function mirroring CesiumJS `createCommand(func, canExecute)`.
pub fn create_command_with_can_execute<Args: Clone + 'static, R: Clone + 'static, F>(
    func: F,
    can_execute: bool,
) -> Command<Args, R>
where
    F: Fn(Args) -> R + 'static,
{
    Command::new_with_can_execute(func, can_execute)
}
