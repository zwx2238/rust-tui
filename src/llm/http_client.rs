use bytes::Bytes;
use rig::http_client::{
    HeaderMap, HeaderValue, HttpClientExt, LazyBody, MultipartForm, Request, Response, Result,
    StreamingResponse,
};
use rig::wasm_compat::WasmCompatSend;
use std::future::Future;

#[derive(Clone, Debug, Default)]
pub struct JsonStreamingClient {
    inner: reqwest::Client,
}

fn ensure_json_content_type(headers: &mut HeaderMap) {
    if headers.contains_key("content-type") {
        return;
    }
    headers.insert("content-type", HeaderValue::from_static("application/json"));
}

impl HttpClientExt for JsonStreamingClient {
    fn send<T, U>(
        &self,
        req: Request<T>,
    ) -> impl Future<Output = Result<Response<LazyBody<U>>>> + WasmCompatSend + 'static
    where
        T: Into<Bytes>,
        T: WasmCompatSend,
        U: From<Bytes>,
        U: WasmCompatSend + 'static,
    {
        HttpClientExt::send(&self.inner, req)
    }

    fn send_multipart<U>(
        &self,
        req: Request<MultipartForm>,
    ) -> impl Future<Output = Result<Response<LazyBody<U>>>> + WasmCompatSend + 'static
    where
        U: From<Bytes>,
        U: WasmCompatSend + 'static,
    {
        HttpClientExt::send_multipart(&self.inner, req)
    }

    fn send_streaming<T>(
        &self,
        mut req: Request<T>,
    ) -> impl Future<Output = Result<StreamingResponse>> + WasmCompatSend
    where
        T: Into<Bytes>,
    {
        ensure_json_content_type(req.headers_mut());
        HttpClientExt::send_streaming(&self.inner, req)
    }
}
