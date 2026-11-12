use std::cell::Cell;
use std::rc::Rc;

use cesium_viewer::widgets::create_command::{
    create_command, create_command_with_can_execute, Command,
};

#[test]
fn works_with_default_value_of_can_execute() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);
    let command = create_command(move |()| {
        cc.set(cc.get() + 1);
        5
    });

    assert!(command.can_execute());
    assert_eq!(command.call().unwrap(), Some(5));
    assert_eq!(call_count.get(), 1);
}

#[test]
fn throws_when_can_execute_value_is_false() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);
    let command = create_command_with_can_execute(
        move |()| {
            cc.set(cc.get() + 1);
            5
        },
        false,
    );

    assert!(!command.can_execute());
    let err = command.call().unwrap_err();
    assert!(err.message.contains("Cannot execute command, canExecute is false."));
    assert_eq!(call_count.get(), 0);
}

#[test]
fn works_with_custom_can_execute_observable() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);
    let command = create_command_with_can_execute(
        move |()| {
            cc.set(cc.get() + 1);
            5
        },
        false,
    );

    assert!(!command.can_execute());
    assert!(command.call().is_err());
    assert_eq!(call_count.get(), 0);

    command.set_can_execute(true);
    assert!(command.can_execute());
    assert_eq!(command.call().unwrap(), Some(5));
    assert_eq!(call_count.get(), 1);
}

#[test]
fn calls_pre_post_events() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);
    let command: Command<String, i32> = create_command(move |arg: String| {
        cc.set(cc.get() + 1);
        arg.len() as i32
    });

    let before_called = Rc::new(Cell::new(0));
    let bc = Rc::clone(&before_called);
    command.before_execute.add_listener(move |info| {
        bc.set(bc.get() + 1);
        assert_eq!(info.args, "cesium");
        assert!(!info.cancel.get());
    });

    let after_called = Rc::new(Cell::new(0));
    let ac = Rc::clone(&after_called);
    command.after_execute.add_listener(move |res| {
        ac.set(ac.get() + 1);
        assert_eq!(*res, 6);
    });

    let result = command.execute("cesium".to_string()).unwrap();
    assert_eq!(result, Some(6));
    assert_eq!(call_count.get(), 1);
    assert_eq!(before_called.get(), 1);
    assert_eq!(after_called.get(), 1);
}

#[test]
fn cancels_a_command_if_before_execute_sets_cancel_to_true() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);
    let command = create_command(move |()| {
        cc.set(cc.get() + 1);
        42
    });

    let before_called = Rc::new(Cell::new(0));
    let bc = Rc::clone(&before_called);
    command.before_execute.add_listener(move |info| {
        bc.set(bc.get() + 1);
        info.cancel.set(true);
    });

    let after_called = Rc::new(Cell::new(0));
    let ac = Rc::clone(&after_called);
    command.after_execute.add_listener(move |_| {
        ac.set(ac.get() + 1);
    });

    let result = command.call().unwrap();
    assert_eq!(result, None);
    assert_eq!(before_called.get(), 1);
    assert_eq!(call_count.get(), 0);
    assert_eq!(after_called.get(), 0);
}
