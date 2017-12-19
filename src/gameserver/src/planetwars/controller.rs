use futures::{Future, Async, Poll, Stream};
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};

use client_controller::{ClientMessage, Message};
use planetwars::config::Config;
use planetwars::step_lock::StepLock;
use planetwars::pw_controller::PwController;

use slog;

/// The controller forms the bridge between game rules and clients.
/// It is responsible for communications, the control flow, and logging.
pub struct Controller {
    step_lock: StepLock,
    pw_controller: PwController,

    client_msgs: UnboundedReceiver<ClientMessage>,
    logger: slog::Logger,
}


#[derive(Clone)]
pub struct Client {
    pub id: usize,
    pub player_name: String,
    pub handle: UnboundedSender<String>,
}

impl Client {
    pub fn send_msg(&mut self, msg: String) {
        // unbounded channels don't fail
        self.handle.unbounded_send(msg).unwrap();
    }
}

impl Controller {
    // TODO: this method does both controller initialization and game staritng.
    // It would be nice to split these.
    pub fn new(clients: Vec<Client>,
               client_msgs: UnboundedReceiver<ClientMessage>,
               conf: Config, logger: slog::Logger,)
               -> Self
    {
        let mut c = Controller {
            pw_controller: PwController::new(conf, clients, logger.clone()),
            step_lock: StepLock::new(),
            client_msgs,
            logger,
        };
        c.init();
        return c;
    }

    fn init(&mut self) {
        self.pw_controller.init(&mut self.step_lock);
    }


    /// Handle an incoming message.
    fn handle_message(&mut self, client_id: usize, msg: Message) {
        match msg {
            Message::Data(msg) => {
                // TODO: maybe it would be better to log this in the
                // client_controller.
                info!(self.logger, "message received";
                    "client_id" => client_id,
                    "content" => &msg,
                );
                self.step_lock.attach_command(client_id, msg);
            },
            Message::Disconnected => {
                // TODO: should a reason be included here?
                // It might be more useful to have the client controller log
                // disconnect reasons.
                info!(self.logger, "client disconnected";
                    "client_id" => client_id
                );
                self.step_lock.remove(client_id);
                self.pw_controller.handle_disconnect(client_id);
            }
        }
    }
}

impl Future for Controller {
    type Item = Vec<usize>;
    type Error = ();

    fn poll(&mut self) -> Poll<Vec<usize>, ()> {
        loop {
            let msg = try_ready!(self.client_msgs.poll()).unwrap();
            self.handle_message(msg.client_id, msg.message);

            while self.step_lock.is_ready() {
                let msgs = self.step_lock.take_messages();
                self.pw_controller.step(&mut self.step_lock, msgs);

                if let Some(result) = self.pw_controller.outcome() {
                    return Ok(Async::Ready(result));
                }
            }
        }
    }
}