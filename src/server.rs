use std::collections::HashMap;
use crate::messages::{PALETTE_SIZE, PALETTE_DEFAULT, Message, UserJoin, UserLeave, History, BoardConfiguration, Color, BoardFlags};
use crate::client::Client;
use crate::ser::to_bytes;
use std::num::Wrapping;
use crate::error::Error;


#[derive(Clone)]
pub struct User {
    pub username: String,
}

/// Main server object holding everything in place.
pub struct Server {
    boards: HashMap<String, Board>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            boards: HashMap::new(),
        }
    }

    pub fn create(&mut self, name: String) -> &mut Board {
        self.boards.entry(name).or_insert(Board::new())
    }

    pub fn has_board(&self, name: &str) -> bool {
        return self.boards.contains_key(name);
    }

    pub fn find(&mut self, name: &str) -> Option<&mut Board> {
        self.boards.get_mut(name)
    }
}

pub struct Board {
    clients: Vec<Client>,
    last_client_id: Wrapping<u8>,
    last_step_id: usize,
    history: Vec<u8>,
    pub history_size: u16,
    palette: [u32; PALETTE_SIZE],
    background_color: Color,
}

impl Board {
    fn new() -> Self {
        return Board {
            clients: vec![],
            last_client_id: Wrapping(0),
            last_step_id: 0,
            history: vec![],
            history_size: std::u16::MAX,
            palette: PALETTE_DEFAULT,
            background_color: 0,
        };
    }

    pub fn board_flags(&self) -> BoardFlags {
        return BoardFlags::HISTORY_ENABLED;
    }

    pub fn broadcast(&mut self, message: &Vec<u8>) {
        let initial = std::mem::replace(&mut self.clients, vec![]);
        let mut errs = vec![];
        for x in initial {
            if let Err(_) = x.out.send(message.clone()) {
                let leave_message = to_bytes(&Message::UserLeave(UserLeave {
                    user_id: x.board_context.unwrap().board_client_id,
                })).unwrap();
                errs.push(leave_message);
            } else {
                self.clients.push(x);
            }
        }

        for err in errs {
            self.broadcast(&err);
        }
    }

    pub fn add_to_history(&mut self, message: &Vec<u8>) {
        self.history.extend(message)
    }

    pub fn add_client(&mut self, client: &mut Client) -> Result<(), Error> {
        let user = match &client.authenticated_user {
            Some(t) => t,
            None => return Err(Error::Message("user not authenticated".to_string()))
        };

        client.board_context.as_mut().map(|x| x.board_client_id = self.last_client_id.0);

        let join_message = to_bytes(&Message::UserJoin(UserJoin {
            username: user.username.as_str(),
            user_id: self.last_client_id.0,
        })).unwrap();

        info!("Client {} has user_id {}", user.username, self.last_client_id.0);

        self.last_client_id += Wrapping(1);
        self.broadcast(&join_message);
        self.clients.push(client.clone());

        /* send board configuration */
        if let Err(_) = client.out.send(to_bytes(&BoardConfiguration {
            history_size: self.history_size,
            palette: self.palette,
            board_flags: self.board_flags(),
            background: self.background_color,
        }).unwrap()) {
            return Err(Error::Message("cannot send board conf".to_string()));
        }

        /* send history */
        for x in self.history.chunks((1 << 16) - 1) {
            let history = to_bytes(&History { data: x }).unwrap();
            if let Err(_) = client.out.send(history) {
                return Err(Error::Message("cannot send history".to_string()));
            }
        }

        Ok(())
    }
}