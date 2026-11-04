use std::rc::Rc;

/// State of a resource request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum RequestState {
    #[default]
    Unissued = 0,
    Issued = 1,
    Active = 2,
    Received = 3,
    Cancelled = 4,
    Failed = 5,
}

impl RequestState {
    pub const UNISSUED: Self = Self::Unissued;
    pub const ISSUED: Self = Self::Issued;
    pub const ACTIVE: Self = Self::Active;
    pub const RECEIVED: Self = Self::Received;
    pub const CANCELLED: Self = Self::Cancelled;
    pub const FAILED: Self = Self::Failed;
}

/// An enum identifying the type of request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum RequestType {
    Terrain = 0,
    Imagery = 1,
    Tiles3D = 2,
    #[default]
    Other = 3,
}

impl RequestType {
    pub const TERRAIN: Self = Self::Terrain;
    pub const IMAGERY: Self = Self::Imagery;
    pub const TILES3D: Self = Self::Tiles3D;
    pub const OTHER: Self = Self::Other;
}

/// Options for configuring a Request instance.
#[derive(Clone, Default)]
pub struct RequestOptions {
    pub url: Option<String>,
    pub priority: f64,
    pub throttle: bool,
    pub throttle_by_server: bool,
    pub request_type: RequestType,
    pub server_key: Option<String>,
    pub priority_function: Option<Rc<dyn Fn() -> f64>>,
    pub cancel_function: Option<Rc<dyn Fn()>>,
}

/// Stores information for making and prioritizing an asynchronous data request.
pub struct Request {
    pub url: Option<String>,
    pub priority: f64,
    pub throttle: bool,
    pub throttle_by_server: bool,
    pub request_type: RequestType,
    pub server_key: Option<String>,
    pub state: RequestState,
    pub cancelled: bool,
    pub priority_function: Option<Rc<dyn Fn() -> f64>>,
    pub cancel_function: Option<Rc<dyn Fn()>>,
}

impl core::fmt::Debug for Request {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Request")
            .field("url", &self.url)
            .field("priority", &self.priority)
            .field("throttle", &self.throttle)
            .field("throttle_by_server", &self.throttle_by_server)
            .field("request_type", &self.request_type)
            .field("server_key", &self.server_key)
            .field("state", &self.state)
            .field("cancelled", &self.cancelled)
            .finish()
    }
}

impl Clone for Request {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            priority: self.priority,
            throttle: self.throttle,
            throttle_by_server: self.throttle_by_server,
            request_type: self.request_type,
            server_key: self.server_key.clone(),
            state: self.state,
            cancelled: self.cancelled,
            priority_function: self.priority_function.clone(),
            cancel_function: self.cancel_function.clone(),
        }
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.priority == other.priority
            && self.throttle == other.throttle
            && self.throttle_by_server == other.throttle_by_server
            && self.request_type == other.request_type
            && self.server_key == other.server_key
            && self.state == other.state
            && self.cancelled == other.cancelled
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new(RequestOptions::default())
    }
}

impl Request {
    pub fn new(options: RequestOptions) -> Self {
        Self {
            url: options.url,
            priority: options.priority,
            throttle: options.throttle,
            throttle_by_server: options.throttle_by_server,
            request_type: options.request_type,
            server_key: options.server_key,
            state: RequestState::Unissued,
            cancelled: false,
            priority_function: options.priority_function,
            cancel_function: options.cancel_function,
        }
    }

    /// Mark the request as cancelled.
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Duplicates properties of this request into another request, resetting its unissued state.
    pub fn clone_into(&self, target: &mut Request) {
        target.url = self.url.clone();
        target.priority = self.priority;
        target.throttle = self.throttle;
        target.throttle_by_server = self.throttle_by_server;
        target.request_type = self.request_type;
        target.server_key = self.server_key.clone();
        target.state = RequestState::Unissued;
        target.cancelled = false;
        target.priority_function = self.priority_function.clone();
        target.cancel_function = self.cancel_function.clone();
    }
}
