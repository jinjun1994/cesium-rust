use std::sync::OnceLock;

use crate::Credit;

pub const DEFAULT_ION_ACCESS_TOKEN: &str =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6Im01MDBjRTVZclBWTmwycGsiLCJqdGkiOiI3NGFjZmI5MS01MDAwLTQwZTktOTlkMC01OTJkN2E2MmQ5OTgiLCJpZCI6MjU5LCJzdWIiOiJDZXNpdW1KUyIsImlzcyI6Imh0dHBzOi8vYXBpLmNlc2l1bS5jb20iLCJhdWQiOiIxLjE0NiBSZWxlYXNlIC0gRGVsZXRlIG9uIERlY2VtYmVyIDEsIDIwMjYiLCJpYXQiOjE3OTAzMTMyMTJ9.4CskcFdVNW5VgLW2iBnYt2GIW2Yl9Q5u6T8gSRK4rkU";

pub const DEFAULT_ION_SERVER: &str = "https://api.cesium.com/";

static DEFAULT_TOKEN_CREDIT: OnceLock<Credit> = OnceLock::new();

/// Cesium ion configuration and helper functions.
pub struct Ion;

impl Ion {
    /// Gets the default token credit if the provided key matches the default access token.
    pub fn get_default_token_credit(provided_key: Option<&str>) -> Option<Credit> {
        if provided_key != Some(DEFAULT_ION_ACCESS_TOKEN) {
            return None;
        }

        let credit = DEFAULT_TOKEN_CREDIT.get_or_init(|| {
            let message = "<b> \
                This application is using Cesium's default ion access token. Please assign <i>Cesium.Ion.defaultAccessToken</i> \
                with an access token from your ion account before making any Cesium API calls. \
                You can sign up for a free ion account at <a href=\"https://cesium.com\">https://cesium.com</a>.</b>";
            Credit::new(message, Some(true))
        });

        Some(credit.clone())
    }
}
