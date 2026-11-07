use cesium_math::IndexDatatype;
use cesium_renderer::{
    Buffer, BufferOptions, BufferTarget, BufferUsage, IndexBufferOptions,
};

#[test]
fn test_vertex_buffer_creation_validation() {
    // Throws when creating without typedArray or sizeInBytes
    let res = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    });
    assert!(res.is_err());

    // Throws when creating with both typedArray and sizeInBytes
    let res = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 16,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(vec![0u8; 16]),
    });
    assert!(res.is_err());

    // Throws when typedArray is empty
    let res = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(vec![]),
    });
    assert!(res.is_err());
}

#[test]
fn test_creates_vertex_buffer_from_size() {
    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 16,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    })
    .unwrap();

    assert_eq!(buffer.size_in_bytes, 16);
    assert_eq!(buffer.usage, BufferUsage::StaticDraw);
    assert_eq!(buffer.target, BufferTarget::VertexBuffer);
    assert_eq!(buffer.data.len(), 16);
}

#[test]
fn test_creates_vertex_buffer_from_typed_array() {
    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StreamDraw),
        typed_array: Some(data.clone()),
    })
    .unwrap();

    assert_eq!(buffer.size_in_bytes, 8);
    assert_eq!(buffer.usage, BufferUsage::StreamDraw);
    assert_eq!(buffer.data, data);
}

#[test]
fn test_copies_array_to_vertex_buffer() {
    let mut buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 12,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    })
    .unwrap();

    let vertices: Vec<u8> = vec![1, 2, 3, 4];
    assert!(buffer.copy_from_array_view(&vertices, Some(0)).is_ok());
    assert_eq!(&buffer.data[0..4], &[1, 2, 3, 4]);

    let offset_data: Vec<u8> = vec![5, 6, 7, 8];
    assert!(buffer.copy_from_array_view(&offset_data, Some(4)).is_ok());
    assert_eq!(&buffer.data[4..8], &[5, 6, 7, 8]);

    // Fails on empty array view
    assert!(buffer.copy_from_array_view(&[], None).is_err());

    // Fails when exceeding sizeInBytes
    let large_data: Vec<u8> = vec![0u8; 10];
    assert!(buffer.copy_from_array_view(&large_data, Some(4)).is_err());
}

#[test]
fn test_index_buffer_creation_and_properties() {
    // 6 bytes for 3 unsigned shorts (2 bytes each)
    let buffer = Buffer::create_index_buffer(IndexBufferOptions {
        size_in_bytes: 6,
        usage: Some(BufferUsage::StreamDraw),
        typed_array: None,
        index_datatype: IndexDatatype::UNSIGNED_SHORT,
    })
    .unwrap();

    assert_eq!(buffer.size_in_bytes, 6);
    assert_eq!(buffer.usage, BufferUsage::StreamDraw);
    assert_eq!(buffer.target, BufferTarget::IndexBuffer);
    assert_eq!(
        buffer.index_datatype,
        Some(IndexDatatype::UNSIGNED_SHORT)
    );
    assert_eq!(buffer.bytes_per_index(), Some(2));
    assert_eq!(buffer.number_of_indices(), Some(3));
}

#[test]
fn test_creates_index_buffer_from_typed_array() {
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    let bytes: Vec<u8> = indices
        .iter()
        .flat_map(|i| i.to_ne_bytes())
        .collect();

    let buffer = Buffer::create_index_buffer(IndexBufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(bytes),
        index_datatype: IndexDatatype::UNSIGNED_SHORT,
    })
    .unwrap();

    assert_eq!(buffer.size_in_bytes, 12);
    assert_eq!(buffer.bytes_per_index(), Some(2));
    assert_eq!(buffer.number_of_indices(), Some(6));
}

#[test]
fn test_get_buffer_data() {
    let data: Vec<u8> = vec![10, 20, 30, 40];
    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(data),
    })
    .unwrap();

    let mut dest = vec![0u8; 4];
    assert!(buffer.get_buffer_data(&mut dest, None, None, None).is_ok());
    assert_eq!(dest, vec![10, 20, 30, 40]);

    let mut partial = vec![0u8; 2];
    assert!(buffer.get_buffer_data(&mut partial, Some(1), Some(0), Some(2)).is_ok());
    assert_eq!(partial, vec![20, 30]);

    // Error checks
    assert!(buffer.get_buffer_data(&mut dest, Some(5), None, None).is_err());
    assert!(buffer.get_buffer_data(&mut dest, None, Some(5), None).is_err());
    assert!(buffer.get_buffer_data(&mut dest, Some(2), Some(0), Some(4)).is_err());
}

#[test]
fn test_copy_from_buffer() {
    let data1: Vec<u8> = vec![1, 2, 3, 4, 5];
    let mut b1 = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(data1),
    })
    .unwrap();

    let mut b2 = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 5,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    })
    .unwrap();

    // Copy b1 to b2
    assert!(b2.copy_from_buffer(&b1, 0, 0, 5).is_ok());
    assert_eq!(b2.data, vec![1, 2, 3, 4, 5]);

    // Incompatible targets check
    let idx_buffer = Buffer::create_index_buffer(IndexBufferOptions {
        size_in_bytes: 5,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
        index_datatype: IndexDatatype::UNSIGNED_BYTE,
    })
    .unwrap();

    assert!(b2.copy_from_buffer(&idx_buffer, 0, 0, 5).is_err());

    // Zero size
    assert!(b2.copy_from_buffer(&b1, 0, 0, 0).is_err());

    // Out of bounds
    assert!(b2.copy_from_buffer(&b1, 3, 0, 4).is_err());
    assert!(b2.copy_from_buffer(&b1, 0, 3, 4).is_err());

    // Overlapping on self via copy_sub_data
    assert!(b1.copy_sub_data(0, 1, 2).is_err()); // same buffer instance overlapping Err
    assert!(b1.copy_sub_data(0, 2, 2).is_ok()); // non-overlapping same buffer instance OK

    // Overlapping on clone with same ID
    let b1_same_id = b1.clone();
    assert!(b1.copy_from_buffer(&b1_same_id, 0, 1, 2).is_err());
    assert!(b1.copy_from_buffer(&b1_same_id, 0, 2, 2).is_ok());
}

#[test]
fn test_destroys() {
    let mut buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 4,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    })
    .unwrap();

    assert!(!buffer.is_destroyed());
    assert!(buffer.destroy().is_ok());
    assert!(buffer.is_destroyed());

    // Double destroy throws
    assert!(buffer.destroy().is_err());
}

#[test]
fn test_creates_pixel_buffer() {
    let mut buffer = Buffer::create_pixel_buffer(BufferOptions {
        size_in_bytes: 16,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    })
    .unwrap();

    assert_eq!(buffer.target, BufferTarget::PixelBuffer);
    assert_eq!(buffer.size_in_bytes, 16);
    assert_eq!(buffer.usage, BufferUsage::StaticDraw);

    let bytes = vec![9u8; 8];
    assert!(buffer.copy_from_array_view(&bytes, Some(0)).is_ok());
    assert_eq!(&buffer.data[0..8], &[9u8; 8]);
}
