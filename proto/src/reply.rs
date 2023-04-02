use std::fmt;
use std::fmt::{format, Formatter};
use crate::error::ProtocolError;

#[repr(u32)]
#[derive(Clone, PartialEq, Debug)]
pub enum Reply {
    MotdStart(String) = 375,
    Motd(String) = 372,
    MotdEnd = 376,

    ErrGeneric(String, Option<Vec<String>>, String) = 400,
    ErrNoSuchCommand(String) = 421,
    ErrNoNicknameGiven = 431,
    ErrErroneousNickname(String) = 432,
    ErrNicknameInUse(String) = 433,
    ErrNickCollision(String) = 436,
    ErrNotRegistered = 451,
    ErrNeedMoreParams(String) = 461,
    ErrAlreadyRegistered = 462,
}

impl<'a> From<&'a Reply> for String {
    fn from(value: &'a Reply) -> Self {
        match value {
            Reply::MotdStart(server) => format!("375 :- {} Message of the day - ", server),
            Reply::Motd(line) => format!("372 :- {}", line),
            Reply::MotdEnd => "376 :End of /MOTD command".to_string(),

            Reply::ErrGeneric(cmd, subs, message) => format!("400 {} {} :{}", cmd, subs.as_ref().unwrap_or( &vec!("".to_owned())).join(" "), message),
            Reply::ErrNoSuchCommand(cmd) => format!("421 {} :Unknown command", cmd),
            Reply::ErrNoNicknameGiven => "431 :No nickname given".to_string(),
            Reply::ErrErroneousNickname(nick) => format!("432 {} :Erroneous nickname", nick),
            Reply::ErrNicknameInUse(nick) => format!("433 {} :Nickname is already in use", nick),
            Reply::ErrNickCollision(nick) => format!("436 {} :Nickname collision KILL", nick),
            Reply::ErrNotRegistered => "451 :You have not registered".to_string(),
            Reply::ErrNeedMoreParams(cmd) => format!("462 {} :Not enough parameters", cmd),
            Reply::ErrAlreadyRegistered => "462 :You may not reregister".to_string(),
        }
    }
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out: String = self.into();
        write!(f, "{}", out)
    }
}

impl TryFrom<ProtocolError> for Reply {
    type Error = ();

    fn try_from(value: ProtocolError) -> Result<Self, Self::Error> {
        match value {
            ProtocolError::NotEnoughArguments(cmd) => Ok(Reply::ErrNeedMoreParams(cmd)),
            _ => Err(())
        }
    }


}