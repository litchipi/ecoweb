#![allow(dead_code)]

// TODO    Fixup protection middleware
//    Get the hostname from other sources (forward, req headers, etc...)
//    Never blame localhost for spam

use actix_web::error::ErrorForbidden;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::{ready, Ready};
use std::net::IpAddr;
use std::time::{Duration, Instant};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{body::MessageBody, Error};
use futures_util::future::LocalBoxFuture;

use crate::config::Configuration;

pub struct ProtectionMiddlewareBuilder {
    limit_per_sec: usize,
}

impl ProtectionMiddlewareBuilder {
    pub fn new(config: &Configuration) -> ProtectionMiddlewareBuilder {
        ProtectionMiddlewareBuilder {
            limit_per_sec: config.req_limit_per_sec,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ProtectionMiddlewareBuilder
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ProtectionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ProtectionMiddleware {
            service,
            addresses: RwLock::new(BTreeMap::new()),
            limit_per_sec: self.limit_per_sec,
        }))
    }
}

pub struct PastConnections {
    last: Instant,
    count: usize,
    banned_since: Option<Instant>,
    ban_duration: Duration,
}

impl Default for PastConnections {
    fn default() -> Self {
        PastConnections {
            last: Instant::now(),
            count: 0,
            banned_since: None,
            ban_duration: Duration::from_secs(30),
        }
    }
}

impl PastConnections {
    pub fn new_connection(&mut self, limit_per_sec: usize) -> bool {
        let delta = Instant::now() - self.last;
        if delta < Duration::from_secs(1) {
            self.count += 1;
        } else {
            self.count = 0;
            self.last = Instant::now();
        }
        log::debug!(
            "New connection ({} since {:?} ago) > {}",
            self.count,
            delta,
            limit_per_sec,
        );
        self.count < limit_per_sec
    }

    pub fn ban_now(&mut self) {
        self.banned_since = Some(Instant::now());
        self.ban_duration = Duration::from_secs(30);
    }

    pub fn banned(&mut self) -> bool {
        if let Some(ref mut since) = self.banned_since {
            if Instant::now() > (*since + self.ban_duration) {
                false
            } else {
                *since = Instant::now();
                self.ban_duration = self.ban_duration.saturating_add(Duration::from_secs(10));
                log::warn!(
                    "Banned IP tried again, ban duration increased to {:?}",
                    self.ban_duration
                );
                true
            }
        } else {
            false
        }
    }
}

pub struct ProtectionMiddleware<S> {
    service: S,
    limit_per_sec: usize,
    addresses: RwLock<BTreeMap<IpAddr, PastConnections>>,
}

impl<S, B> ProtectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
}

impl<S, B> Service<ServiceRequest> for ProtectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        _: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req.peer_addr().unwrap().ip();
        {
            let mut addr_write = self.addresses.write();
            let md = addr_write.entry(ip).or_insert(PastConnections::default());
            if md.banned() {
                return Box::pin(async move { Err(ErrorForbidden("Banned")) });
            }
            if !md.new_connection(self.limit_per_sec) {
                md.ban_now();
                log::error!("Banning address {ip:?}");
                return Box::pin(async move { Err(ErrorForbidden("Banned")) });
            }
        }
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
