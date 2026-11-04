use cesium_math::Heap;
use std::cmp::Ordering;

fn check_heap<T, F>(heap: &Heap<T, F>, comparator: &F) -> bool
where
    F: Fn(&T, &T) -> Ordering,
{
    let array = heap.internal_array();
    let length = heap.len();
    for i in 0..length {
        let left = 2 * (i + 1) - 1;
        let right = 2 * (i + 1);
        if left < length && comparator(&array[i], &array[left]).is_gt() {
            return false;
        }
        if right < length && comparator(&array[i], &array[right]).is_gt() {
            return false;
        }
    }
    true
}

#[test]
fn test_heap_maintains_property_on_insert_and_pop() {
    let comparator = |a: &f64, b: &f64| a.partial_cmp(b).unwrap();
    let mut heap = Heap::new(comparator);

    let length = 100;
    for i in 0..length {
        let val = ((i * 37 + 13) % 100) as f64;
        heap.insert(val);
        assert!(check_heap(&heap, &comparator));
    }

    for _ in 0..length {
        heap.pop(0);
        assert!(check_heap(&heap, &comparator));
    }
}

#[test]
fn test_heap_limited_by_maximum_length() {
    let comparator = |a: &f64, b: &f64| a.partial_cmp(b).unwrap();
    let max_len = 50;
    let mut heap = Heap::with_maximum_length(comparator, max_len);

    let length = 100;
    for i in 0..length {
        let val = ((i * 47 + 7) % 100) as f64;
        heap.insert(val);
        assert!(check_heap(&heap, &comparator));
        assert!(heap.len() <= max_len);
    }
}

#[test]
fn test_heap_pops_in_sorted_order() {
    let comparator = |a: &f64, b: &f64| a.partial_cmp(b).unwrap();
    let mut heap = Heap::new(comparator);

    let length = 100;
    for i in 0..length {
        let val = ((i * 73 + 19) % 100) as f64;
        heap.insert(val);
    }

    let mut curr = heap.pop(0).unwrap();
    for _ in 0..(length - 1) {
        let next = heap.pop(0).unwrap();
        assert!(comparator(&curr, &next).is_le());
        curr = next;
    }
}

#[test]
fn test_heap_insert_returns_removed_element_when_full() {
    let comparator = |a: &f64, b: &f64| a.partial_cmp(b).unwrap();
    let max_len = 10;
    let mut heap = Heap::with_maximum_length(comparator, max_len);

    for i in 0..max_len {
        let removed = heap.insert(i as f64);
        assert_eq!(removed, None);
    }

    // Insert larger value than anything inside (since it's min-heap, max value ends up at the bottom)
    let removed = heap.insert(100.0);
    assert_eq!(removed, Some(100.0));

    // Insert smaller value, which should push out a larger value
    let removed = heap.insert(-1.0);
    assert!(removed.is_some());
    assert_ne!(removed, Some(-1.0));
}

#[test]
fn test_heap_resort() {
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Item {
        distance: f64,
        id: usize,
    }

    let comparator = |a: &Item, b: &Item| a.distance.partial_cmp(&b.distance).unwrap();
    let mut heap = Heap::new(comparator);

    let length = 50;
    for i in 0..length {
        heap.insert(Item {
            distance: i as f64 / (length as f64 - 1.0),
            id: i,
        });
    }

    let mut elements = Vec::new();
    let mut current_id = 0;
    while let Some(element) = heap.pop(0) {
        elements.push(element);
        assert!(element.id >= current_id);
        current_id = element.id;
    }

    // Invert distances and reinsert
    for el in &mut elements {
        el.distance = 1.0 - el.distance;
        heap.insert(*el);
    }

    // Resort
    heap.resort();

    let mut current_id = length;
    while let Some(element) = heap.pop(0) {
        assert!(element.id <= current_id);
        current_id = element.id;
    }
}
