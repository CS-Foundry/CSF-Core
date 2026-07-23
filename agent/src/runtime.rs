use anyhow::Result;
use axum::body::Bytes;
use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::spec::WorkloadSpec;

pub type LogStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub struct ExecSession {
    pub input: Pin<Box<dyn AsyncWrite + Send>>,
    pub output: Pin<Box<dyn AsyncRead + Send>>,
}

pub struct StreamAsyncRead<S> {
    stream: Pin<Box<S>>,
    pending: Bytes,
}

impl<S> StreamAsyncRead<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>>,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: Box::pin(stream),
            pending: Bytes::new(),
        }
    }
}

impl<S> AsyncRead for StreamAsyncRead<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>>,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            if !self.pending.is_empty() {
                let take = self.pending.len().min(buf.remaining());
                let chunk = self.pending.split_to(take);
                buf.put_slice(&chunk);
                return Poll::Ready(Ok(()));
            }

            match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.pending = chunk;
                    continue;
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(e)),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContainerStats {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_bytes: Option<i64>,
    pub network_rx_bytes: Option<i64>,
    pub network_tx_bytes: Option<i64>,
}

#[async_trait::async_trait]
pub trait Runtime: Send + Sync {
    async fn pull_image(&self, image: &str) -> Result<()>;
    async fn start_workload(&self, spec: &WorkloadSpec) -> Result<String>;
    async fn inspect_status(&self, workload_handle: &str) -> Result<String>;
    fn logs(&self, workload_handle: &str) -> LogStream;
    async fn exec(&self, workload_handle: &str) -> Result<ExecSession>;
    async fn stop_workload(&self, workload_handle: &str) -> Result<()>;
    async fn stats(&self, workload_handle: &str) -> Result<ContainerStats>;

    async fn service_network_ip(
        &self,
        _workload_handle: &str,
        _network_name: &str,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    async fn list_managed_workloads(&self) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
}
