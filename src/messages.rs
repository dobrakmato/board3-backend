use serde::{Serialize, Deserialize};
use bitflags::bitflags;

const fn color(r: u8, g: u8, b: u8) -> u32 {
    return (b as u32) << 16 | (g as u32) << 8 | r as u32;
}

pub const PALETTE_SIZE: usize = 8;
pub const PALETTE_DEFAULT: Palette = [
    color(0, 0, 0),
    color(255, 0, 0),
    color(0, 255, 0),
    color(0, 0, 255),
    color(255, 255, 0),
    color(0, 255, 255),
    color(255, 0, 255),
    color(255, 255, 255)
];

/* types */
pub type Palette = [u32; PALETTE_SIZE];
pub type Color = u8;
pub type Position = u32;
pub type UserId = u8;
pub type StepId = u32;

/* custom types */
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct BoardFlags: u8 {
        const HISTORY_ENABLED = 0b00000001;
        const HISTORY_TRIMMED = 0b00000010;
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct DrawFlags(pub u8);

/* messages */

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Auth<'a> {
    pub jwt_token: &'a str
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Create<'a> {
    pub template_id: u64,
    pub name: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Join<'a> {
    pub name: &'a str
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct BoardConfiguration {
    pub palette: Palette,
    pub background: Color,
    pub board_flags: BoardFlags,
    pub history_size: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Step {
    pub step_id: StepId
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Draw {
    pub position: Position,
    pub color: Color,
    pub flags: DrawFlags,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CursorMove {
    pub position: Position,
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Fill {
    pub start: Position,
    pub end: Position,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Image<'a> {
    pub start: Position,
    pub end: Position,
    pub url: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Text<'a> {
    pub center: Position,
    pub text: &'a str,
    pub text_color: Color,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Undo {
    pub last_actual_step_id: StepId
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Ping {
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct UserJoin<'a> {
    pub user_id: UserId,
    pub username: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct UserLeave {
    pub user_id: UserId
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ServerMessage<'a> {
    pub message: &'a str
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct History<'a> {
    pub data: &'a [u8]
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Message<'a> {
    #[serde(borrow)]
    Auth(Auth<'a>),
    Join(Join<'a>),
    Create(Create<'a>),
    BoardConfiguration(BoardConfiguration),
    Step(Step),
    Draw(Draw),
    CursorMove(CursorMove),
    Fill(Fill),
    Image(Image<'a>),
    Text(Text<'a>),
    Undo(Undo),
    Ping(Ping),
    UserJoin(UserJoin<'a>),
    UserLeave(UserLeave),
    ServerMessage(ServerMessage<'a>),
    History(History<'a>),
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;
    use crate::messages::{Message, Auth, Join, Create, BoardConfiguration, PALETTE_SIZE, BoardFlags, Step, StepId, Draw, Position, Color, DrawFlags, CursorMove, UserId, Fill, Image, Text, Undo, Ping, UserJoin, UserLeave, ServerMessage, History};
    use crate::ser::to_bytes;
    use crate::de::from_bytes;
    use rand::Rng;


    #[quickcheck]
    fn test_auth(token: String) -> bool {
        let message = Message::Auth(Auth {
            jwt_token: token.as_str()
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_join(name: String) -> bool {
        let message = Message::Join(Join {
            name: name.as_str()
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_create(name: String, template_id: u64) -> bool {
        let message = Message::Create(Create {
            template_id,
            name: name.as_str(),
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_board_configuration(history_size: u16, board_flags2: u8, background: u8) -> bool {
        let mut rng = rand::thread_rng();
        let mut palette = [0; PALETTE_SIZE];
        palette.iter_mut().map(|x| *x = rng.gen());
        let message = Message::BoardConfiguration(BoardConfiguration {
            palette,
            background,
            board_flags: BoardFlags::from_bits_truncate(board_flags2),
            history_size,
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_step(step_id: StepId) -> bool {
        let message = Message::Step(Step {
            step_id
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_draw(position: Position, color: Color, flags2: u8) -> bool {
        let message = Message::Draw(Draw {
            position,
            color,
            flags: DrawFlags(flags2),
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_cursor_move(position: Position, user_id: UserId) -> bool {
        let message = Message::CursorMove(CursorMove {
            position,
            user_id,
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_fill(start: Position, end: Position, color: Color) -> bool {
        let message = Message::Fill(Fill {
            start,
            end,
            color,
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_image(start: Position, end: Position, color: Color, url: String) -> bool {
        let message = Message::Image(Image {
            start,
            end,
            url: url.as_str(),
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_text(center: Position, text_color: Color, text: String) -> bool {
        let message = Message::Text(Text {
            center,
            text: text.as_str(),
            text_color,
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_undo(last_actual_step_id: StepId) -> bool {
        let message = Message::Undo(Undo {
            last_actual_step_id
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_ping(timestamp: u64) -> bool {
        let message = Message::Ping(Ping {
            timestamp
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_user_join(user_id: UserId, username: String) -> bool {
        let message = Message::UserJoin(UserJoin {
            user_id,
            username: username.as_str(),
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_user_leave(user_id: UserId) -> bool {
        let message = Message::UserLeave(UserLeave {
            user_id,
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_server_message(message: String) -> bool {
        let message = Message::ServerMessage(ServerMessage {
            message: message.as_str()
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }

    #[quickcheck]
    fn test_history(data: Vec<u8>) -> bool {
        let message = Message::History(History {
            data: data.as_slice()
        });
        let serialized = to_bytes(&message).unwrap();
        let deserialized: Message = from_bytes(serialized.as_slice()).unwrap();
        return message == deserialized;
    }
}
