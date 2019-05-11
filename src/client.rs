use ws::{Sender, Error, Handler, CloseCode, Message, ErrorKind};
use crate::messages::{Message as ObMessage, Create, Join};
use crate::de::from_bytes;
use crate::server::User;
use crate::server::Server;
use std::cell::RefCell;
use crate::auth::auth;
use log::{info, warn};

thread_local! {
    static SERVER: RefCell<Server> = RefCell::new(Server::new());
}

#[derive(Clone)]
pub struct BoardContext {
    pub board_name: String,
    pub board_client_id: u8,
}

#[derive(Clone)]
pub struct Client {
    pub out: Sender,
    pub authenticated_user: Option<User>,
    pub board_context: Option<BoardContext>,
}

impl Handler for Client {
    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(_) => return self.out.close_with_reason(CloseCode::Invalid, "expected binary"),
            Message::Binary(t) => self.handle_binary_msg(t)
        }
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        match code {
            CloseCode::Normal => info!("The client is done with the connection."),
            CloseCode::Away => info!("The client is leaving the site."),
            _ => warn!("The client encountered an error: {}", reason),
        }
    }
}

impl Client {
    fn handle_binary_msg(&mut self, t: Vec<u8>) -> Result<(), Error> {
        let msg: ObMessage = match from_bytes(t.as_slice()) {
            Ok(t) => t,
            Err(_) => return self.out.close_with_reason(CloseCode::Error, "invalid message"),
        };

        /* check auth */
        if self.authenticated_user.is_none() {
            return self.ensure_auth(msg);
        }

        /* check room join */
        if self.board_context.is_none() {
            return self.ensure_in_board(msg);
        }

        /* handler other cases */
        self.handle_in_board_msg(msg, &t)
    }

    fn ensure_auth(&mut self, msg: ObMessage) -> Result<(), Error> {
        match msg {
            ObMessage::Auth(t) => match auth(t) {
                None => return self.out.close_with_reason(CloseCode::Error, "invalid auth"),
                Some(t) => {
                    self.authenticated_user = Some(t);
                    info!("Client {} authenticated successfully", t.username);
                    Ok(())
                }
            },
            _ => return self.out.close_with_reason(CloseCode::Error, "auth expected"),
        }
    }

    fn ensure_in_board(&mut self, msg: ObMessage) -> Result<(), Error> {
        match msg {
            ObMessage::Join(t) => self.handle_board_join(t),
            ObMessage::Create(t) => self.handle_board_create(t),
            _ => return self.out.close_with_reason(CloseCode::Error, "auth expected"),
        }
    }

    fn handle_board_create(&mut self, t: Create) -> Result<(), Error> {
        return SERVER.with(|x| {
            let mut server = x.borrow_mut();

            if server.has_board(t.name) { return self.out.close_with_reason(CloseCode::Error, "board already exists"); }

            self.board_context = Some(BoardContext {
                board_client_id: 0,
                board_name: String::from(t.name),
            });

            info!("Client {} is creating board {} (template_id={})", self.authenticated_user.unwrap().username, t.name, t.template_id);
            server.create(String::from(t.name))
                .add_client(self)
                .map_err(|_| ws::Error::new(ErrorKind::Internal, "cannot add client to board"))
        });
    }

    fn handle_board_join(&mut self, t: Join) -> Result<(), Error> {
        return SERVER.with(|x| {
            let mut server = x.borrow_mut();
            match server.find(t.name) {
                Some(b) => {
                    info!("Client {} is creating board {}", self.authenticated_user.unwrap().username, t.name);
                    b.add_client(self)
                        .map_err(|_| ws::Error::new(ErrorKind::Internal, "cannot add client to board"))
                }
                None => self.out.close_with_reason(CloseCode::Error, "board not found"),
            }
        });
    }

    fn handle_in_board_msg(&mut self, msg: ObMessage, t: &Vec<u8>) -> Result<(), Error> {
        match msg {
            ObMessage::Auth(_) => self.out.close_with_reason(CloseCode::Error, "already authenticated"),
            ObMessage::Join(_) => self.out.close_with_reason(CloseCode::Error, "already joined a board"),
            ObMessage::BoardConfiguration(_) => self.out.close_with_reason(CloseCode::Error, "board configuration invalid atm"),
            ObMessage::History(_) => self.out.close_with_reason(CloseCode::Error, "history invalid atm"),
            ObMessage::ServerMessage(_) => self.out.close_with_reason(CloseCode::Error, "server message invalid atm"),
            ObMessage::UserJoin(_) => self.out.close_with_reason(CloseCode::Error, "user join invalid atm"),
            ObMessage::UserLeave(_) => self.out.close_with_reason(CloseCode::Error, "user leave invalid atm"),
            ObMessage::Create(_) => self.out.close_with_reason(CloseCode::Error, "already joined a board"),
            ObMessage::Ping(_) => { Ok(()) }
            ObMessage::Step(_) => { Ok(()) }
            ObMessage::Undo(_) => { Ok(()) }
            ObMessage::Draw(_) | ObMessage::CursorMove(_) | ObMessage::Fill(_) | ObMessage::Image(_) | ObMessage::Text(_) => self.broadcast_to_board(t),
        }
    }

    fn broadcast_to_board(&mut self, t: &Vec<u8>) -> Result<(), Error> {
        SERVER.with(|x| {
            let mut server = x.borrow_mut();

            self.board_context.as_ref().map(|x| {
                let board = server.find(&x.board_name).unwrap();

                if board.history_size != 0 {
                    board.add_to_history(t);
                }

                board.broadcast(t)
            });
        });
        Ok(())
    }
}
