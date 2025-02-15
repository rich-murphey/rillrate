use super::link;
use crate::actors::router::Router;
use crate::config::ServerConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, StartedBy};
use meio_connect::server::{HttpServer, HttpServerLink};
use rill_client::actors::broadcaster::Broadcaster;

/// Embedded node.
pub struct RillServer {
    server_config: ServerConfig,
    public_server: Option<HttpServerLink>,
    private_server: Option<HttpServerLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Tuning,
    Broadcaster,
    HttpServer,
    Endpoints,
}

impl Actor for RillServer {
    type GroupBy = Group;
}

impl RillServer {
    /// Create a new instance of an embedded node.
    pub fn new(server_config: Option<ServerConfig>) -> Self {
        Self {
            server_config: server_config.unwrap_or_default(),
            public_server: None,
            private_server: None,
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillServer {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Tuning,
            Group::Broadcaster,
            Group::HttpServer,
            Group::Endpoints,
        ]);

        // Starting all basic actors
        let extern_addr = format!("{}:{}", self.server_config.server_address(), 9090).parse()?;
        let extern_http_server_actor = HttpServer::new(extern_addr);
        let extern_http_server = ctx.spawn_actor(extern_http_server_actor, Group::HttpServer);
        self.public_server = Some(extern_http_server.link());

        let inner_addr = format!("127.0.0.1:{}", 0).parse()?;
        let inner_http_server_actor = HttpServer::new(inner_addr);
        let inner_http_server = ctx.spawn_actor(inner_http_server_actor, Group::HttpServer);
        self.private_server = Some(inner_http_server.link());

        let exporter_actor = Broadcaster::new();
        let exporter = ctx.spawn_actor(exporter_actor, Group::Broadcaster);

        let server_actor = Router::new(
            inner_http_server.link(),
            extern_http_server.link(),
            exporter.link(),
        );
        let _router = ctx.spawn_actor(server_actor, Group::Endpoints);

        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for RillServer {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Broadcaster> for RillServer {
    async fn handle(
        &mut self,
        _id: IdOf<Broadcaster>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("Broadcaster finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<HttpServer> for RillServer {
    async fn handle(
        &mut self,
        _id: IdOf<HttpServer>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("HttpServer finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Router> for RillServer {
    async fn handle(&mut self, _id: IdOf<Router>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Router finished");
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::WaitPublicEndpoint> for RillServer {
    async fn handle(
        &mut self,
        _msg: link::WaitPublicEndpoint,
        _ctx: &mut Context<Self>,
    ) -> Result<HttpServerLink, Error> {
        // `public_server` always available here since it's attached in `StartedBy` handler
        self.public_server
            .clone()
            .ok_or_else(|| Error::msg("no public server"))
    }
}

#[async_trait]
impl InteractionHandler<link::WaitPrivateEndpoint> for RillServer {
    async fn handle(
        &mut self,
        _msg: link::WaitPrivateEndpoint,
        _ctx: &mut Context<Self>,
    ) -> Result<HttpServerLink, Error> {
        // `private_server` always available here since it's attached in `StartedBy` handler
        self.private_server
            .clone()
            .ok_or_else(|| Error::msg("no private server"))
    }
}
