use tokio::{spawn, sync::mpsc};

use crate::{app::App, interface::Interface, server::Server};

mod app;
mod database;
mod interface;
mod protocol;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let (client_send_for_app, recv_for_server) = mpsc::channel(10);
    let (send_for_server, recv_for_app) = mpsc::channel(10);
    let (interface_send_for_app, recv_for_interface) = mpsc::channel(10);
    let send_for_interface = send_for_server.clone();
    let send_for_protocol = client_send_for_app.clone();

    spawn(async move {
        App::new(interface_send_for_app, client_send_for_app, recv_for_app)
            .start()
            .await
    });

    spawn(async move {
        Server::new(send_for_server, send_for_protocol, recv_for_server)
            .await?
            .start()
            .await
    });

    let mut interface = Interface::new(send_for_interface, recv_for_interface).await?;
    interface.start().await
}
