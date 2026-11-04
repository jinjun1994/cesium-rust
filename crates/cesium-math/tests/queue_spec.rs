use cesium_math::Queue;

#[test]
fn test_queue_enqueue_dequeue() {
    let mut queue = Queue::new();
    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
    assert_eq!(queue.dequeue(), None);

    queue.enqueue("a".to_string());
    queue.enqueue("b".to_string());
    queue.enqueue("c".to_string());

    assert_eq!(queue.len(), 3);
    assert!(!queue.is_empty());
    assert_eq!(queue.peek(), Some(&"a".to_string()));

    assert_eq!(queue.dequeue(), Some("a".to_string()));
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.dequeue(), Some("b".to_string()));
    assert_eq!(queue.dequeue(), Some("c".to_string()));
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn test_queue_contains_and_clear() {
    let mut queue = Queue::new();
    queue.enqueue(1);
    queue.enqueue(2);

    assert!(queue.contains(&1));
    assert!(queue.contains(&2));
    assert!(!queue.contains(&3));

    queue.clear();
    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
    assert_eq!(queue.peek(), None);
}

#[test]
fn test_queue_sort() {
    let mut queue = Queue::new();
    queue.enqueue(3);
    queue.enqueue(1);
    queue.enqueue(2);

    queue.sort_by(|a, b| a.cmp(b));

    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), Some(3));
}
