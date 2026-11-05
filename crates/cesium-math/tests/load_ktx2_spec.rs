use cesium_math::{load_ktx2, load_ktx2_url, KTX2SupportedFormats, KTX2Transcoder};

static GREEN_4X4_ETC1S_KTX2: &[u8] =
    include_bytes!("../../../../cesium/Specs/Data/Images/Green4x4_ETC1S.ktx2");
static LOGO_32X32_UASTC_ZSTD_KTX2: &[u8] =
    include_bytes!("../../../../cesium/Specs/Data/Images/Logo32x32_UASTC_Zstd.ktx2");

#[test]
fn test_throws_with_no_buffer() {
    let result = std::panic::catch_unwind(|| {
        // load_ktx2 with None
    });
    assert!(result.is_ok());
    // Directly check load_ktx2(None, None)
    let rt = tokio_less_block_on(load_ktx2(None, None));
    assert!(rt.is_err());
}

#[test]
fn test_throws_with_unknown_supported_formats() {
    let result = KTX2Transcoder::transcode(&[], None);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("supportedTargetFormats is required"));
}

#[test]
fn test_throws_if_called_with_invalid_url() {
    let test_url = "http://example.invalid/testuri";
    let result = load_ktx2_url(test_url, Some(KTX2SupportedFormats { etc: true, ..Default::default() }));
    assert!(result.is_err());
}

fn expect_ktx2_transcode_result(
    buffer: &[u8],
    supported_formats: KTX2SupportedFormats,
    width: u32,
    height: u32,
    is_compressed: bool,
) {
    let result = KTX2Transcoder::transcode(buffer, Some(supported_formats)).unwrap();
    assert_eq!(result.width, width);
    assert_eq!(result.height, height);
    assert_eq!(result.internal_format.is_compressed_format(), is_compressed);
    assert!(!result.buffer_view.is_empty());
}

#[test]
fn test_transcodes_etc1s_ktx2_to_etc() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { etc: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_etc() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { etc: true, ..Default::default() },
        32,
        32,
        true,
    );
}

#[test]
fn test_transcodes_etc1s_ktx2_to_etc1() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { etc1: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_etc1() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { etc1: true, ..Default::default() },
        32,
        32,
        true,
    );
}

#[test]
fn test_transcodes_etc1s_ktx2_to_astc() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { astc: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_astc() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { astc: true, ..Default::default() },
        32,
        32,
        true,
    );
}

#[test]
fn test_transcodes_etc1s_ktx2_to_bc7() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { bc7: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_bc7() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { bc7: true, ..Default::default() },
        32,
        32,
        true,
    );
}

#[test]
fn test_transcodes_etc1s_ktx2_to_s3tc() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { s3tc: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_s3tc() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { s3tc: true, ..Default::default() },
        32,
        32,
        true,
    );
}

#[test]
fn test_transcodes_etc1s_ktx2_to_pvrtc() {
    expect_ktx2_transcode_result(
        GREEN_4X4_ETC1S_KTX2,
        KTX2SupportedFormats { pvrtc: true, ..Default::default() },
        4,
        4,
        true,
    );
}

#[test]
fn test_transcodes_uastc_ktx2_to_pvrtc() {
    expect_ktx2_transcode_result(
        LOGO_32X32_UASTC_ZSTD_KTX2,
        KTX2SupportedFormats { pvrtc: true, ..Default::default() },
        32,
        32,
        true,
    );
}

fn tokio_less_block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    let mut fut = Box::pin(fut);
    unsafe fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    match Pin::as_mut(&mut fut).poll(&mut cx) {
        Poll::Ready(res) => res,
        Poll::Pending => panic!("Future did not complete immediately"),
    }
}
