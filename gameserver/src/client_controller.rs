use futures::{Future, Poll, Async};
use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use futures::stream::{Stream, SplitStream, SplitSink};
use tokio_io::AsyncRead;
use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use bytes::{BytesMut, BufMut};
use std::io;
use std::str;
use std::process;
use slog;

use bot_runner::BotHandle;
use buffered_sender::BufferedSender;
use planetwars::controller::PlayerId;


error_chain! {
    errors {
        ConnectionClosed
    }

    foreign_links {
        Io(io::Error);
    }
}

pub struct ClientMessage {
    pub player_id: PlayerId,
    pub message: Message,
}

pub enum Message {
    Data(String),
    Connected,
    Disconnected,
    Timeout,
}

#[derive(PartialEq)]
enum Connection {
    Connected,
    Disconnected,
}


// TODO: the client controller should also be handed a log handle

pub struct ClientController {
    player_id: PlayerId,
    player_name: String,
    
    sender: BufferedSender<SplitSink<Transport>>,
    client_msgs: SplitStream<Transport>,

    ctrl_chan: UnboundedReceiver<String>,
    ctrl_handle: UnboundedSender<String>,
    
    game_handle: UnboundedSender<ClientMessage>,

    logger: slog::Logger,
    connected: Connection,
}

impl ClientController {
    pub fn new(player_id: PlayerId,
               player_name: String,
               conn: BotHandle,
               game_handle: UnboundedSender<ClientMessage>,
               logger: &slog::Logger)
               -> Self
    {
        let (snd, rcv) = unbounded();
        let (sink, stream) = conn.framed(LineCodec).split();

        let c = ClientController {
            sender: BufferedSender::new(sink),
            client_msgs: stream,

            ctrl_chan: rcv,
            ctrl_handle: snd,

            game_handle,
            player_id,
            player_name,

            logger: logger.new(
                o!("player_id" => player_id.as_usize())
            ),
            connected: Connection::Disconnected,
        };
        return c;
    }

    /// Get a handle to the control channel for this client.
    pub fn handle(&self) -> UnboundedSender<String> {
        self.ctrl_handle.clone()
    }

    /// Send a message to the game this controller serves.
    fn send_message(&mut self, message: Message) {
        let msg = ClientMessage {
            player_id: self.player_id.clone(),
            message: message,
        };
        self.game_handle.unbounded_send(msg).expect("game handle broke");
    }

    /// Pull messages from the client, and handle them.
    fn handle_client_msgs(&mut self) -> Poll<(), Error> {
        while let Some(line) = try_ready!(self.client_msgs.poll()) {
            let msg = Message::Data(line);
            self.send_message(msg);
        }
        bail!(ErrorKind::ConnectionClosed)
    }

    /// The unit error type of ctrl_chan.poll() means that it won't error. Since
    /// we can't cast "won't error" to our custom error type, we cannot use the
    /// try_ready! macro with polling ith ctrl_chan. This method provides an
    /// adapter to Poll with our error type.
    fn poll_ctrl_chan(&mut self) -> Poll<Option<String>, Error> {
        let res = self.ctrl_chan.poll();
        Ok(res.unwrap())
    }

    /// Pull commands from the control channel, and handle them. Note: for now
    /// this should never error, but once we actually handle commands errors
    /// might be possible.
    fn handle_commands(&mut self) -> Poll<(), Error> {
        while let Some(command) = try_ready!(self.poll_ctrl_chan()) {
            self.sender.send(command);
        }
        // Since we entirely control this channel, it should not fail.
        // If it does, something is very wrong and we should find out what
        // to do about that.
        panic!("Command handle broke");
    }

    /// Try sending messages to the client, in an asynchronous fashion.
    fn write_messages(&mut self) -> Poll<(), io::Error> {
        self.sender.poll()
    }

    /// Step the future, allowing errors to be thrown.
    /// These errors then get handled in the actual poll implementation.
    fn try_poll(&mut self) -> Poll<(), Error> {
        try!(self.handle_commands());

        if self.connected == Connection::Disconnected {
            self.connected = Connection::Connected;
            self.send_message(Message::Connected);
        }

        try!(self.handle_client_msgs());
        try!(self.write_messages());
        // TODO: returning NotReady unconditionally here might be a little
        // dodgy.
        Ok(Async::NotReady)
    }
}

impl Future for ClientController {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        // TODO: errors should be handled
        Ok(match self.try_poll() {
            // Returning ready terminates the task. We only want to do that when
            // an error happens.
            // TODO: how do we handle graceful disconnects?
            Ok(Async::Ready(())) => panic!("something bad happened"),
            Ok(Async::NotReady) => Async::NotReady,
            Err(_err) => {
                eprintln!("[GAMESERVER] Failed to communicate with bot \"{}\".", self.player_name);
                eprintln!("[GAMESERVER] Hopefully, an error message from the offending bot can be found above.");
                eprintln!("[GAMESERVER] Game terminating; all other bots will now be killed.");
                // TODO: are some errors recoverable?
                // should they be handled in this method, or ad-hoc?
                // TODO: log the actual error
                // Let the game know this client failed.
                // self.send_message(Message::Disconnected);
                // TODO: what do when this fails?
                // self.write_messages().expect("could not write messages");
                // terminate the program
                process::exit(1);
            }
        })
    }
}



// This is rather temporary.
type Transport = Framed<BotHandle, LineCodec>;

struct LineCodec;

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        buf.reserve(msg.len() + 1);
        buf.extend(msg.as_bytes());
        buf.put_u8(b'\n');
        Ok(())
    }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        // Check to see if the frame contains a new line
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            // remove line from buffer
            let line = buf.split_to(n);

            // remove newline
            buf.split_to(1);

            // Try to decode the line as UTF-8
            return match str::from_utf8(&line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::Other, "invalid string")
                ),
            }
        }

        Ok(None)
    }
}
