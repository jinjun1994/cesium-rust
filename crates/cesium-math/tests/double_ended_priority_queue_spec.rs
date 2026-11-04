use cesium_math::DoubleEndedPriorityQueue;

#[test]
fn test_double_ended_priority_queue_basic() {
    let mut queue = DoubleEndedPriorityQueue::new(|a: &i32, b: &i32| a.cmp(b));
    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
    assert_eq!(queue.get_minimum(), None);
    assert_eq!(queue.get_maximum(), None);

    queue.insert(1);
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.get_minimum(), Some(&1));
    assert_eq!(queue.get_maximum(), Some(&1));

    queue.insert(2);
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.get_minimum(), Some(&1));
    assert_eq!(queue.get_maximum(), Some(&2));

    queue.insert(3);
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.get_minimum(), Some(&1));
    assert_eq!(queue.get_maximum(), Some(&3));

    queue.insert(4);
    assert_eq!(queue.len(), 4);
    assert_eq!(queue.get_minimum(), Some(&1));
    assert_eq!(queue.get_maximum(), Some(&4));
}

#[test]
fn test_double_ended_priority_queue_remove_min_max() {
    let mut queue = DoubleEndedPriorityQueue::new(|a: &i32, b: &i32| a.cmp(b));
    for x in [5, 3, 8, 1, 9, 2, 7, 4, 6] {
        queue.insert(x);
    }
    assert_eq!(queue.len(), 9);

    assert_eq!(queue.remove_minimum(), Some(1));
    assert_eq!(queue.remove_maximum(), Some(9));
    assert_eq!(queue.remove_minimum(), Some(2));
    assert_eq!(queue.remove_maximum(), Some(8));
}

#[test]
fn test_double_ended_priority_queue_maximum_length() {
    let mut queue = DoubleEndedPriorityQueue::with_maximum_length(|a: &i32, b: &i32| a.cmp(b), 3);
    assert_eq!(queue.maximum_length(), Some(3));

    queue.insert(10);
    queue.insert(20);
    queue.insert(30);

    // Inserting a smaller element than minimum (5 <= 10) returns 5 and is not added
    let rejected = queue.insert(5);
    assert_eq!(rejected, Some(5));
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.get_minimum(), Some(&10));

    // Inserting a larger element (25 > 10) removes minimum (10) and inserts 25
    let dropped = queue.insert(25);
    assert_eq!(dropped, Some(10));
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.get_minimum(), Some(&20));
    assert_eq!(queue.get_maximum(), Some(&30));
}
