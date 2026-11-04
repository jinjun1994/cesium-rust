use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::event::Event;
use crate::heap::Heap;
use crate::request::{Request, RequestState};

/// Request scheduler statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RequestSchedulerStatistics {
    pub number_of_attempted_requests: usize,
    pub number_of_active_requests: usize,
    pub number_of_cancelled_requests: usize,
    pub number_of_cancelled_active_requests: usize,
    pub number_of_failed_requests: usize,
    pub number_of_active_requests_ever: usize,
    pub last_number_of_active_requests: usize,
}

#[inline]
pub fn is_data_uri(url: &str) -> bool {
    url.starts_with("data:")
}

#[inline]
pub fn is_blob_uri(url: &str) -> bool {
    url.starts_with("blob:")
}

/// Derive the server key (`host:port`) from a given URL.
pub fn get_server_key(url_str: &str) -> Result<String, &'static str> {
    if url_str.is_empty() {
        return Err("url is required");
    }

    let parsed = if let Ok(u) = url::Url::parse(url_str) {
        u
    } else {
        url::Url::parse(&format!("http://{}", url_str)).map_err(|_| "invalid url")?
    };

    let scheme = parsed.scheme();
    let host = parsed.host_str().ok_or("url must have a host")?;
    let port = parsed
        .port()
        .unwrap_or(if scheme == "https" { 443 } else { 80 });
    Ok(format!("{}:{}", host, port))
}

type RequestComparator = Box<dyn Fn(&Rc<RefCell<Request>>, &Rc<RefCell<Request>>) -> std::cmp::Ordering>;

fn sort_requests(a: &Rc<RefCell<Request>>, b: &Rc<RefCell<Request>>) -> std::cmp::Ordering {
    let pa = a.borrow().priority;
    let pb = b.borrow().priority;
    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
}

/// The request scheduler is used to track and constrain the number of active requests in order to prioritize incoming requests.
pub struct RequestScheduler {
    pub maximum_requests: usize,
    pub maximum_requests_per_server: usize,
    pub requests_by_server: HashMap<String, usize>,
    pub throttle_requests: bool,
    pub debug_show_statistics: bool,
    priority_heap_length: usize,
    pub statistics: RequestSchedulerStatistics,
    pub request_completed_event: Event<Option<String>>,
    active_requests: Vec<Rc<RefCell<Request>>>,
    number_of_active_requests_by_server: HashMap<String, usize>,
    request_heap: Heap<Rc<RefCell<Request>>, RequestComparator>,
}

impl Default for RequestScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestScheduler {
    pub fn new() -> Self {
        let comparator: RequestComparator = Box::new(sort_requests);
        let mut heap = Heap::with_maximum_length(comparator, 20);
        heap.reserve(20);
        Self {
            maximum_requests: 50,
            maximum_requests_per_server: 18,
            requests_by_server: HashMap::new(),
            throttle_requests: true,
            debug_show_statistics: false,
            priority_heap_length: 20,
            statistics: RequestSchedulerStatistics::default(),
            request_completed_event: Event::new(),
            active_requests: Vec::new(),
            number_of_active_requests_by_server: HashMap::new(),
            request_heap: heap,
        }
    }

    #[inline]
    pub fn priority_heap_length(&self) -> usize {
        self.priority_heap_length
    }

    pub fn set_priority_heap_length(&mut self, value: usize) {
        if value < self.priority_heap_length {
            while self.request_heap.len() > value {
                if let Some(req) = self.request_heap.pop(0) {
                    self.cancel_request(&req);
                }
            }
        }
        self.priority_heap_length = value;
        self.request_heap.set_maximum_length(Some(value));
        self.request_heap.reserve(value);
    }

    pub fn server_has_open_slots(&self, server_key: &str, desired_requests: usize) -> bool {
        let max_requests = self
            .requests_by_server
            .get(server_key)
            .copied()
            .unwrap_or(self.maximum_requests_per_server);
        let active = self
            .number_of_active_requests_by_server
            .get(server_key)
            .copied()
            .unwrap_or(0);
        active + desired_requests <= max_requests
    }

    pub fn heap_has_open_slots(&self, desired_requests: usize) -> bool {
        self.request_heap.len() + desired_requests <= self.priority_heap_length
    }

    pub fn number_of_active_requests_by_server(&self, server_key: &str) -> usize {
        self.number_of_active_requests_by_server
            .get(server_key)
            .copied()
            .unwrap_or(0)
    }

    pub fn cancel_request(&mut self, request_ref: &Rc<RefCell<Request>>) {
        let is_active = {
            let mut req = request_ref.borrow_mut();
            let active = req.state == RequestState::Active;
            req.state = RequestState::Cancelled;
            if let Some(ref cancel_fn) = req.cancel_function {
                cancel_fn();
            }
            active
        };

        self.statistics.number_of_cancelled_requests += 1;

        if is_active {
            self.statistics.number_of_active_requests =
                self.statistics.number_of_active_requests.saturating_sub(1);
            if let Some(ref server_key) = request_ref.borrow().server_key {
                if let Some(count) = self.number_of_active_requests_by_server.get_mut(server_key) {
                    *count = count.saturating_sub(1);
                }
            }
            self.statistics.number_of_cancelled_active_requests += 1;
        }
    }

    fn start_request(&mut self, request_ref: &Rc<RefCell<Request>>) -> bool {
        {
            let mut req = request_ref.borrow_mut();
            req.state = RequestState::Active;
        }
        self.active_requests.push(request_ref.clone());
        self.statistics.number_of_active_requests += 1;
        self.statistics.number_of_active_requests_ever += 1;
        if let Some(ref server_key) = request_ref.borrow().server_key {
            *self
                .number_of_active_requests_by_server
                .entry(server_key.clone())
                .or_insert(0) += 1;
        }
        true
    }

    fn update_priority(&self, request_ref: &Rc<RefCell<Request>>) {
        let mut req = request_ref.borrow_mut();
        if let Some(ref p_fn) = req.priority_function {
            req.priority = p_fn();
        }
    }

    /// Issue a request through the scheduler.
    pub fn request(&mut self, request_ref: &Rc<RefCell<Request>>) -> Result<bool, &'static str> {
        let url = {
            let req = request_ref.borrow();
            match &req.url {
                Some(u) => u.clone(),
                None => return Err("request.url is required"),
            }
        };

        if is_data_uri(&url) || is_blob_uri(&url) {
            self.request_completed_event.raise_event(&None);
            request_ref.borrow_mut().state = RequestState::Received;
            return Ok(true);
        }

        self.statistics.number_of_attempted_requests += 1;

        let server_key = {
            let mut req = request_ref.borrow_mut();
            if req.server_key.is_none() {
                let key = get_server_key(&url)?;
                req.server_key = Some(key.clone());
                key
            } else {
                req.server_key.clone().unwrap()
            }
        };

        let (throttle, throttle_by_server) = {
            let req = request_ref.borrow();
            (req.throttle, req.throttle_by_server)
        };

        if self.throttle_requests
            && throttle_by_server
            && !self.server_has_open_slots(&server_key, 1)
        {
            return Ok(false);
        }

        if !self.throttle_requests || !throttle {
            return Ok(self.start_request(request_ref));
        }

        if self.active_requests.len() >= self.maximum_requests {
            return Ok(false);
        }

        self.update_priority(request_ref);
        let removed = self.request_heap.insert(request_ref.clone());

        if let Some(removed_req) = removed {
            if Rc::ptr_eq(&removed_req, request_ref) {
                return Ok(false);
            }
            self.cancel_request(&removed_req);
        }

        request_ref.borrow_mut().state = RequestState::Issued;
        Ok(true)
    }

    /// Complete a request successfully.
    pub fn complete_request(&mut self, request_ref: &Rc<RefCell<Request>>) {
        let (server_key, is_cancelled) = {
            let req = request_ref.borrow();
            (req.server_key.clone(), req.state == RequestState::Cancelled)
        };
        if is_cancelled {
            return;
        }

        self.statistics.number_of_active_requests =
            self.statistics.number_of_active_requests.saturating_sub(1);
        if let Some(ref key) = server_key {
            if let Some(count) = self.number_of_active_requests_by_server.get_mut(key) {
                *count = count.saturating_sub(1);
            }
        }
        self.request_completed_event.raise_event(&None);
        request_ref.borrow_mut().state = RequestState::Received;
    }

    /// Mark a request as failed.
    pub fn fail_request(&mut self, request_ref: &Rc<RefCell<Request>>, error_message: String) {
        let (server_key, is_cancelled) = {
            let req = request_ref.borrow();
            (req.server_key.clone(), req.state == RequestState::Cancelled)
        };
        if is_cancelled {
            return;
        }

        self.statistics.number_of_failed_requests += 1;
        self.statistics.number_of_active_requests =
            self.statistics.number_of_active_requests.saturating_sub(1);
        if let Some(ref key) = server_key {
            if let Some(count) = self.number_of_active_requests_by_server.get_mut(key) {
                *count = count.saturating_sub(1);
            }
        }
        self.request_completed_event.raise_event(&Some(error_message));
        request_ref.borrow_mut().state = RequestState::Failed;
    }

    /// Update the request scheduler by promoting queued requests and retiring inactive ones.
    pub fn update(&mut self) {
        let active_snapshot = self.active_requests.clone();
        for req_ref in &active_snapshot {
            if req_ref.borrow().cancelled {
                self.cancel_request(req_ref);
            }
        }
        self.active_requests
            .retain(|req_ref| req_ref.borrow().state == RequestState::Active);

        for req_ref in self.request_heap.internal_array() {
            self.update_priority(req_ref);
        }
        self.request_heap.resort();

        let open_slots = self
            .maximum_requests
            .saturating_sub(self.active_requests.len());
        let mut filled_slots = 0;
        while filled_slots < open_slots && !self.request_heap.is_empty() {
            let request_ref = match self.request_heap.pop(0) {
                Some(r) => r,
                None => break,
            };

            if request_ref.borrow().cancelled {
                self.cancel_request(&request_ref);
                continue;
            }

            let throttle_by_server = request_ref.borrow().throttle_by_server;
            let server_key = request_ref.borrow().server_key.clone().unwrap_or_default();

            if throttle_by_server && !self.server_has_open_slots(&server_key, 1) {
                self.cancel_request(&request_ref);
                continue;
            }

            self.start_request(&request_ref);
            filled_slots += 1;
        }

        self.update_statistics();
    }

    fn update_statistics(&mut self) {
        if !self.debug_show_statistics {
            return;
        }
        self.statistics.last_number_of_active_requests = self.statistics.number_of_active_requests;
    }

    #[inline]
    pub fn request_heap(&self) -> &Heap<Rc<RefCell<Request>>, RequestComparator> {
        &self.request_heap
    }

    #[inline]
    pub fn request_heap_mut(&mut self) -> &mut Heap<Rc<RefCell<Request>>, RequestComparator> {
        &mut self.request_heap
    }

    pub fn clear_for_specs(&mut self) {
        self.maximum_requests = 50;
        self.maximum_requests_per_server = 18;
        self.requests_by_server.clear();
        self.throttle_requests = true;
        self.debug_show_statistics = false;
        self.priority_heap_length = 20;
        self.statistics = RequestSchedulerStatistics::default();
        self.request_completed_event = Event::new();
        self.active_requests.clear();
        self.number_of_active_requests_by_server.clear();
        let comparator: RequestComparator = Box::new(sort_requests);
        let mut heap = Heap::with_maximum_length(comparator, 20);
        heap.reserve(20);
        self.request_heap = heap;
    }
}
