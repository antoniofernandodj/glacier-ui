//! An HTTP client for Rust, on the **same network layer** the Luau `fetch` uses: the same
//! cookie jar, the same `multipart/form-data`, the same cancellation. It is what a
//! [`Component`](crate::Component) written in Rust calls from inside `ctx.perform(async { … })`.
//!
//! There are two levels:
//!
//! - [`Request`]: one request, built and sent (what `fetch(url, opts)` is in Luau);
//! - [`Client`]: a reusable object configured once — base URL, default headers and parameters,
//!   timeout, retries, interceptors — with one shortcut per method
//!   ([`get`](Client::get), [`post`](Client::post), [`put`](Client::put), [`patch`](Client::patch),
//!   [`delete`](Client::delete), [`head`](Client::head), [`options`](Client::options) and
//!   [`query`](Client::query)). It is the Rust counterpart of the Luau `http()` client.
//!
//! ```ignore
//! use glacier_ui::net::Client;
//!
//! let api = Client::new("https://api.example.com")
//!     .default_header("Authorization", "Bearer …")
//!     .default_param("version", "1")
//!     .retries(2);
//!
//! let res = api.get("/items").param("page", "2").send().await;
//! if res.ok() {
//!     let items = res.json();
//! }
//! let created = api.post("/items").json(&serde_json::json!({ "name": "x" })).send().await;
//! ```
//!
//! # The `QUERY` method (RFC 10008)
//!
//! [`Client::query`] sends the HTTP `QUERY` method: like `GET` it is **safe and idempotent**, but
//! the query travels in the request **body**, as with `POST`. It is for queries too large or too
//! structured for a URL.
//!
//! ```ignore
//! let res = api
//!     .query("/search")
//!     .json(&serde_json::json!({ "term": "glacier", "limit": 10 }))
//!     .send()
//!     .await;
//! ```
//!
//! RFC 10008 §2 says a server must reject a `QUERY` whose `Content-Type` is missing or disagrees
//! with the content, so the client refuses to send one without a body and a `Content-Type`
//! (`.json(…)` sets it): the error comes back as a [`Response`] with `ok() == false`, without
//! touching the network. Being idempotent, a `QUERY` is retried like a `GET` (see below).
//!
//! # Retries
//!
//! `retries(n)` repeats a request up to `n` extra times on a network failure (status 0), `429`
//! or `5xx` — **but only for idempotent methods** (`GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE`,
//! `QUERY`). A `POST` or `PATCH` is never repeated on its own: the first attempt may have
//! reached the server. [`Client::retry_on`] replaces the rule. A cancelled request is never
//! repeated.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::component::{AbortHandle, FetchResult, Multipart, PendingFetch, Progress, ProgressFn};

// ───────────────────────────────────── Request ─────────────────────────────────────

/// One HTTP request, built and sent. The Rust counterpart of Luau's `fetch(url, opts)`.
pub struct Request(PendingFetch);

impl Request {
    pub fn new(url: impl Into<String>, method: &str) -> Self {
        Self(PendingFetch::new(0, url.into(), method.to_string(), None, Vec::new()))
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(url, "GET")
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(url, "POST")
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.headers.push((name.into(), value.into()));
        self
    }

    /// A text body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.0.body = Some(body.into());
        self
    }

    /// The raw contents of a file as the body, read when the request is sent (no base64, no
    /// copy through a `String`). Wins over [`body`](Request::body).
    pub fn body_file(mut self, path: impl Into<String>) -> Self {
        self.0.body_file = Some(path.into());
        self
    }

    /// Follow up to `n` redirects (301/302/303/307/308). `0` (the default) returns the 3xx as is.
    /// Never goes from https to http, and a hop to another host drops `Authorization` and `Cookie`.
    pub fn max_redirects(mut self, n: u32) -> Self {
        self.0.max_redirects = n;
        self
    }

    /// A ceiling for the response body, in bytes: past it the request fails (`ok = false`)
    /// without filling memory or disk.
    pub fn max_bytes(mut self, n: u64) -> Self {
        self.0.max_bytes = Some(n);
        self
    }

    /// A ceiling for the whole request, in milliseconds.
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.0.timeout_ms = Some(ms);
        self
    }

    /// Ignore the cookie jar for this request: nothing is sent and nothing is stored.
    pub fn no_cookies(mut self) -> Self {
        self.0.cookies = false;
        self
    }

    /// `handle.abort()` cancels the request (`ok = false`, `error = "cancelado"`).
    pub fn abortable(mut self, handle: &AbortHandle) -> Self {
        self.0.abort = Some(handle.clone());
        self
    }

    /// Send `multipart/form-data`; the files are read from disk when sending.
    pub fn multipart(mut self, multipart: Multipart) -> Self {
        self.0.multipart = Some(multipart);
        self
    }

    /// Called while the request body is sent and the response body is received (about 20
    /// times a second at most, plus once when each phase ends), from the network task — not
    /// the UI thread. Hand the numbers to the UI through a channel or an `Arc<Mutex<…>>`.
    pub fn on_progress(mut self, f: impl Fn(Progress) + Send + Sync + 'static) -> Self {
        self.0.progress = Some(ProgressFn::new(f));
        self
    }

    /// Write the raw response body to this path (for binaries); the result's `body` is empty.
    pub fn download_to(mut self, path: impl Into<String>) -> Self {
        self.0.download_to = Some(path.into());
        self
    }

    /// Send and wait. Never fails: a network error becomes `ok = false`.
    pub async fn send(self) -> FetchResult {
        crate::net::perform(self.0).await
    }
}

// ────────────────────────────────────── Client ─────────────────────────────────────

/// The pieces of a request a client builds up — what an `on_request` hook may change.
#[derive(Debug, Clone, Default)]
pub struct RequestParts {
    pub method: String,
    /// A path appended to the client's base URL, or an absolute `http(s)://` URL.
    pub path: String,
    pub headers: Vec<(String, String)>,
    /// Query-string parameters (`?a=1&b=2`), added after the client's default ones.
    pub params: Vec<(String, String)>,
    pub body: Option<String>,
    /// A file whose raw contents are the body (see [`Request::body_file`]).
    pub body_file: Option<String>,
    pub multipart: Option<Multipart>,
    pub abort: Option<AbortHandle>,
    /// `None` = the client's setting.
    pub cookies: Option<bool>,
    /// `None` = the client's setting.
    pub timeout_ms: Option<u64>,
    pub progress: Option<ProgressFn>,
}

/// A response: the raw result plus the decoded JSON, when the body is JSON.
#[derive(Debug, Clone)]
pub struct Response {
    pub raw: FetchResult,
    json: Option<Value>,
    /// The URL that was requested (with the query string).
    pub url: String,
    pub method: String,
    /// How many attempts it took (1 = no retry).
    pub attempts: u32,
    /// Free-form notes hooks can leave for the caller (`on_response`).
    pub meta: HashMap<String, String>,
}

impl Response {
    /// `true` for a 2xx status.
    pub fn ok(&self) -> bool {
        self.raw.ok
    }

    /// The HTTP status, or 0 when the request never got a response.
    pub fn status(&self) -> u16 {
        self.raw.status
    }

    pub fn text(&self) -> &str {
        &self.raw.body
    }

    /// The error message (empty when the request got a response, even a 4xx/5xx one).
    pub fn error(&self) -> &str {
        &self.raw.error
    }

    /// The body decoded as JSON, or `None` if it was not JSON.
    pub fn json(&self) -> Option<&Value> {
        self.json.as_ref()
    }
}

/// What an `on_error` hook receives: a request that failed for good (after the retries).
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub status: u16,
    pub error: String,
    pub url: String,
    pub method: String,
    pub attempts: u32,
}

type RequestHook = Arc<dyn Fn(&mut RequestParts) + Send + Sync>;
type ResponseHook = Arc<dyn Fn(&mut Response) + Send + Sync>;
type ErrorHook = Arc<dyn Fn(&ErrorInfo) + Send + Sync>;
type RetryRule = Arc<dyn Fn(&Response) -> bool + Send + Sync>;

/// A reusable HTTP client. Configure it once, then call one of the method shortcuts. See the
/// module docs.
#[derive(Clone)]
pub struct Client {
    base: String,
    headers: Vec<(String, String)>,
    params: Vec<(String, String)>,
    timeout_ms: Option<u64>,
    retries: u32,
    retry_on: Option<RetryRule>,
    cookies: bool,
    on_request: Vec<RequestHook>,
    on_response: Vec<ResponseHook>,
    on_error: Vec<ErrorHook>,
}

fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

fn has_header(headers: &[(String, String)], name: &str) -> bool {
    headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
}

/// Methods a retry can repeat without changing the outcome (RFC 9110 §9.2.2, RFC 10008 §2).
fn is_idempotent(method: &str) -> bool {
    matches!(
        method.to_ascii_uppercase().as_str(),
        "GET" | "HEAD" | "OPTIONS" | "PUT" | "DELETE" | "QUERY"
    )
}

impl Client {
    /// A client for `base_url`; every path is appended to it.
    pub fn new(base_url: &str) -> Self {
        Self {
            base: base_url.trim_end_matches('/').to_string(),
            headers: Vec::new(),
            params: Vec::new(),
            timeout_ms: None,
            retries: 0,
            retry_on: None,
            cookies: true,
            on_request: Vec::new(),
            on_response: Vec::new(),
            on_error: Vec::new(),
        }
    }

    // — configuration (chainable) —

    /// A header sent with every request (a request's own header of the same name is added after).
    pub fn default_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// A query-string parameter sent with every request.
    pub fn default_param(mut self, name: &str, value: &str) -> Self {
        self.params.push((name.into(), value.into()));
        self
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Extra attempts on network failure, `429` and `5xx` — idempotent methods only.
    pub fn retries(mut self, n: u32) -> Self {
        self.retries = n;
        self
    }

    /// Replaces the retry rule: return `true` to try again. It still never repeats a
    /// cancelled request, and still stops after [`retries`](Client::retries).
    pub fn retry_on(mut self, rule: impl Fn(&Response) -> bool + Send + Sync + 'static) -> Self {
        self.retry_on = Some(Arc::new(rule));
        self
    }

    /// This client ignores the cookie jar (a request can still turn it on).
    pub fn no_cookies(mut self) -> Self {
        self.cookies = false;
        self
    }

    /// Runs before each request is sent, in registration order; may change anything.
    pub fn on_request(mut self, hook: impl Fn(&mut RequestParts) + Send + Sync + 'static) -> Self {
        self.on_request.push(Arc::new(hook));
        self
    }

    /// Runs after each response (the last attempt), before `on_error`.
    pub fn on_response(mut self, hook: impl Fn(&mut Response) + Send + Sync + 'static) -> Self {
        self.on_response.push(Arc::new(hook));
        self
    }

    /// Runs when a request ends not-ok, after the retries.
    pub fn on_error(mut self, hook: impl Fn(&ErrorInfo) + Send + Sync + 'static) -> Self {
        self.on_error.push(Arc::new(hook));
        self
    }

    /// A derived client: it inherits everything (hooks included) and its base URL grows by `path`.
    pub fn extend(&self, path: &str) -> Self {
        let mut c = self.clone();
        c.base = format!("{}/{}", self.base, path.trim_matches('/'));
        c
    }

    /// Forget the cookies stored for this client's host.
    pub fn clear_cookies(&self) {
        let host = self
            .base
            .split_once("://")
            .map(|(_, rest)| rest.split('/').next().unwrap_or(rest).to_string());
        crate::net::clear_cookies(host.as_deref());
    }

    // — one shortcut per method —

    /// A request with any method.
    pub fn request(&self, method: &str, path: &str) -> RequestBuilder<'_> {
        RequestBuilder {
            client: self,
            parts: RequestParts {
                method: method.to_ascii_uppercase(),
                path: path.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder<'_> {
        self.request("GET", path)
    }

    pub fn post(&self, path: &str) -> RequestBuilder<'_> {
        self.request("POST", path)
    }

    pub fn put(&self, path: &str) -> RequestBuilder<'_> {
        self.request("PUT", path)
    }

    pub fn patch(&self, path: &str) -> RequestBuilder<'_> {
        self.request("PATCH", path)
    }

    pub fn delete(&self, path: &str) -> RequestBuilder<'_> {
        self.request("DELETE", path)
    }

    pub fn head(&self, path: &str) -> RequestBuilder<'_> {
        self.request("HEAD", path)
    }

    pub fn options(&self, path: &str) -> RequestBuilder<'_> {
        self.request("OPTIONS", path)
    }

    /// The HTTP `QUERY` method (RFC 10008): a safe, idempotent request whose query travels in the
    /// body. Needs a body **and** a `Content-Type` — `.json(…)` sets both.
    pub fn query(&self, path: &str) -> RequestBuilder<'_> {
        self.request("QUERY", path)
    }

    // — the engine —

    async fn execute(&self, mut p: RequestParts) -> Response {
        for hook in &self.on_request {
            hook(&mut p);
        }

        let mut params = self.params.clone();
        params.extend(p.params.clone());
        let qs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect();
        let path = if p.path.starts_with("http://") || p.path.starts_with("https://") {
            p.path.clone()
        } else if p.path.starts_with('/') || p.path.is_empty() {
            format!("{}{}", self.base, p.path)
        } else {
            format!("{}/{}", self.base, p.path)
        };
        let url = if qs.is_empty() {
            path
        } else {
            format!("{path}{}{}", if path.contains('?') { '&' } else { '?' }, qs.join("&"))
        };
        let mut headers = self.headers.clone();
        headers.extend(p.headers.clone());

        let fail = |error: String, attempts: u32| Response {
            raw: FetchResult::error(error),
            json: None,
            url: url.clone(),
            method: p.method.clone(),
            attempts,
            meta: HashMap::new(),
        };

        // RFC 10008 §2: a QUERY without content, or without an accurate Content-Type, is a request
        // the server must reject. Do not send it.
        if p.method == "QUERY" && p.multipart.is_none() {
            if p.body.as_deref().is_none_or(str::is_empty) && p.body_file.is_none() {
                let r = fail(
                    "QUERY needs a request body (the query itself) — RFC 10008 §2".into(),
                    0,
                );
                return self.finish(r);
            }
            if !has_header(&headers, "content-type") {
                let r = fail(
                    "QUERY needs a Content-Type header (e.g. use `.json(…)`) — RFC 10008 §2".into(),
                    0,
                );
                return self.finish(r);
            }
        }

        let cookies = p.cookies.unwrap_or(self.cookies);
        let timeout = p.timeout_ms.or(self.timeout_ms);
        let mut attempts = 0u32;
        let mut response;
        loop {
            attempts += 1;
            let mut req = Request::new(url.clone(), &p.method);
            for (k, v) in &headers {
                req = req.header(k.clone(), v.clone());
            }
            if let Some(body) = &p.body {
                req = req.body(body.clone());
            }
            if let Some(path) = &p.body_file {
                req = req.body_file(path.clone());
            }
            if let Some(ms) = timeout {
                req = req.timeout_ms(ms);
            }
            if !cookies {
                req = req.no_cookies();
            }
            if let Some(h) = &p.abort {
                req = req.abortable(h);
            }
            if let Some(m) = &p.multipart {
                req = req.multipart(m.clone());
            }
            if let Some(f) = &p.progress {
                req.0.progress = Some(f.clone());
            }
            let raw = req.send().await;
            let json = serde_json::from_str::<Value>(&raw.body).ok();
            response = Response {
                raw,
                json,
                url: url.clone(),
                method: p.method.clone(),
                attempts,
                meta: HashMap::new(),
            };
            let cancelled = response.raw.error == FetchResult::cancelado().error;
            let retry = if cancelled || response.raw.ok || attempts > self.retries {
                false
            } else if let Some(rule) = &self.retry_on {
                rule(&response)
            } else {
                is_idempotent(&p.method)
                    && (response.raw.status == 0
                        || response.raw.status == 429
                        || response.raw.status >= 500)
            };
            if !retry {
                break;
            }
        }
        self.finish(response)
    }

    fn finish(&self, mut response: Response) -> Response {
        for hook in &self.on_response {
            hook(&mut response);
        }
        if !response.raw.ok {
            let info = ErrorInfo {
                status: response.raw.status,
                error: if response.raw.error.is_empty() {
                    format!("HTTP {}", response.raw.status)
                } else {
                    response.raw.error.clone()
                },
                url: response.url.clone(),
                method: response.method.clone(),
                attempts: response.attempts,
            };
            for hook in &self.on_error {
                hook(&info);
            }
        }
        response
    }
}

/// A request being built by a [`Client`] shortcut; finish it with [`send`](RequestBuilder::send).
pub struct RequestBuilder<'c> {
    client: &'c Client,
    parts: RequestParts,
}

impl RequestBuilder<'_> {
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.parts.headers.push((name.into(), value.into()));
        self
    }

    /// A query-string parameter (`?name=value`), percent-encoded.
    pub fn param(mut self, name: &str, value: &str) -> Self {
        self.parts.params.push((name.into(), value.into()));
        self
    }

    /// A text body. Set a `Content-Type` yourself (or use [`json`](RequestBuilder::json)).
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.parts.body = Some(body.into());
        self
    }

    /// The raw contents of a file as the body (see [`Request::body_file`]).
    pub fn body_file(mut self, path: impl Into<String>) -> Self {
        self.parts.body_file = Some(path.into());
        self
    }

    /// A JSON body, with `Content-Type: application/json` (unless one was already set).
    pub fn json(mut self, value: &Value) -> Self {
        self.parts.body = Some(value.to_string());
        if !has_header(&self.parts.headers, "content-type") {
            self.parts
                .headers
                .push(("Content-Type".into(), "application/json".into()));
        }
        self
    }

    /// `multipart/form-data`, using the client's base URL and headers.
    pub fn multipart(mut self, multipart: Multipart) -> Self {
        self.parts.multipart = Some(multipart);
        self
    }

    /// `handle.abort()` cancels this call (which is then never retried).
    pub fn abort(mut self, handle: &AbortHandle) -> Self {
        self.parts.abort = Some(handle.clone());
        self
    }

    /// See [`Request::on_progress`].
    pub fn on_progress(mut self, f: impl Fn(Progress) + Send + Sync + 'static) -> Self {
        self.parts.progress = Some(ProgressFn::new(f));
        self
    }

    /// Turn the cookie jar on or off for this call, whatever the client says.
    pub fn cookies(mut self, on: bool) -> Self {
        self.parts.cookies = Some(on);
        self
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.parts.timeout_ms = Some(ms);
        self
    }

    pub async fn send(self) -> Response {
        self.client.execute(self.parts).await
    }
}
