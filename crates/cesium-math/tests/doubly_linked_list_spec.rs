use cesium_math::DoublyLinkedList;

#[test]
fn test_doubly_linked_list_add_and_remove() {
    let mut list = DoublyLinkedList::new();
    assert_eq!(list.head, None);
    assert_eq!(list.tail, None);
    assert_eq!(list.len(), 0);

    let n1 = list.add(1);
    assert_eq!(list.head, Some(n1));
    assert_eq!(list.tail, Some(n1));
    assert_eq!(list.len(), 1);

    let node1 = list.node(n1).unwrap();
    assert_eq!(node1.item, 1);
    assert_eq!(node1.previous, None);
    assert_eq!(node1.next, None);

    let n2 = list.add(2);
    assert_eq!(list.head, Some(n1));
    assert_eq!(list.tail, Some(n2));
    assert_eq!(list.len(), 2);
    assert_eq!(list.node(n1).unwrap().next, Some(n2));
    assert_eq!(list.node(n2).unwrap().previous, Some(n1));

    let n3 = list.add(3);
    assert_eq!(list.head, Some(n1));
    assert_eq!(list.tail, Some(n3));
    assert_eq!(list.len(), 3);
    assert_eq!(list.node(n2).unwrap().next, Some(n3));
    assert_eq!(list.node(n3).unwrap().previous, Some(n2));

    // Remove middle (n2)
    assert_eq!(list.remove(n2), Some(2));
    assert_eq!(list.len(), 2);
    assert_eq!(list.head, Some(n1));
    assert_eq!(list.tail, Some(n3));
    assert_eq!(list.node(n1).unwrap().next, Some(n3));
    assert_eq!(list.node(n3).unwrap().previous, Some(n1));

    // Remove head (n1)
    assert_eq!(list.remove(n1), Some(1));
    assert_eq!(list.len(), 1);
    assert_eq!(list.head, Some(n3));
    assert_eq!(list.tail, Some(n3));

    // Remove remaining (n3)
    assert_eq!(list.remove(n3), Some(3));
    assert_eq!(list.len(), 0);
    assert_eq!(list.head, None);
    assert_eq!(list.tail, None);
}

#[test]
fn test_doubly_linked_list_splice() {
    let mut list = DoublyLinkedList::new();
    let _n1 = list.add(1);
    let n2 = list.add(2);
    let _n3 = list.add(3);
    let n4 = list.add(4);
    let n5 = list.add(5);

    // Initial order: 1, 2, 3, 4, 5
    // Splice n2 after n4 -> expected order: 1, 3, 4, 2, 5
    list.splice(n4, n2);

    assert_eq!(list.len(), 5);
    let mut current = list.head;
    let mut items = Vec::new();
    while let Some(curr_id) = current {
        let node = list.node(curr_id).unwrap();
        items.push(node.item);
        current = node.next;
    }
    assert_eq!(items, vec![1, 3, 4, 2, 5]);
    assert_eq!(list.tail, Some(n5));
}
