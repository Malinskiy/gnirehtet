use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io;
use std::rc::Rc;
use mio::{Event, PollOpt, Ready};
use mio::tcp::TcpListener;

use super::client::Client;
use super::selector::Selector;

const TAG: &'static str = "TunnelServer";

pub struct TunnelServer {
    clients: Vec<Rc<RefCell<Client>>>,
    tcp_listener: TcpListener,
    next_client_id: u32,
}

impl TunnelServer {
    pub fn new(port: u16, selector: &mut Selector) -> io::Result<Rc<RefCell<Self>>> {
        let tcp_listener = TunnelServer::start_socket(port)?;
        let rc = Rc::new(RefCell::new(Self {
            clients: Vec::new(),
            tcp_listener: tcp_listener,
            next_client_id: 0,
        }));
        let rc_clone = rc.clone();
        let handler = Rc::new(move |selector: &mut Selector, ready| {
            let mut self_ref = rc_clone.borrow_mut();
            self_ref.on_ready(selector, ready);
        });
        selector.register(&rc.borrow().tcp_listener, handler, Ready::readable(), PollOpt::edge())?;
        Ok(rc)
    }

    fn start_socket(port: u16) -> io::Result<TcpListener> {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(localhost, port);
        let server = TcpListener::bind(&addr)?;
        Ok(server)
    }

    fn accept_client(&mut self, selector: &mut Selector) -> io::Result<()> {
        let (stream, _) = self.tcp_listener.accept()?;
        let client_id = self.next_client_id;
        self.next_client_id += 1;
        let client = Client::new(client_id, selector, stream)?;
        self.clients.push(client);
        info!(target: TAG, "Client #{} connected", client_id);
        Ok(())
    }

    fn on_ready(&mut self, selector: &mut Selector, _: Event) {
        if let Err(err) = self.accept_client(selector) {
            error!(target: TAG, "Cannot accept client: {}", err);
        }
    }
}