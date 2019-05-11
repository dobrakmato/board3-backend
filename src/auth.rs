use crate::messages::Auth;
use crate::server::User;

pub fn auth(auth: Auth) -> Option<User> {
    // todo: actually perform authentication
    Some(User {
        username: auth.jwt_token.to_string()
    })
}
