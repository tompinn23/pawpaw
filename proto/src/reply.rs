use std::fmt;
use std::fmt::Formatter;
use crate::command::CommandError;

#[repr(u32)]
#[derive(Clone, PartialEq, Debug)]
pub enum Reply {
    ErrNoSuchCommand(String) = 421,
    ErrNickCollision(String) = 436,
    ErrNotRegistered = 451,
    ErrNeedMoreParams(String) = 461,
}

impl<'a> From<&'a Reply> for String {
    fn from(value: &'a Reply) -> Self {
        match value {
            Reply::ErrNoSuchCommand(cmd) => format!("421 {} :Unknown command", cmd),
            Reply::ErrNickCollision(nick) => format!("436 {} :Nickname collision KILL", nick),
            Reply::ErrNotRegistered => "451 :You have not registered".to_string(),
            Reply::ErrNeedMoreParams(cmd) => format!("462 {} :Not enough parameters", cmd),
        }
    }
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out: String = self.into();
        write!(f, "{}", out)
    }
}

impl TryFrom<CommandError> for Reply {
    type Error = ();

    fn try_from(value: CommandError) -> Result<Self, Self::Error> {
        match value {
            CommandError::NotEnoughArguments(cmd) => Ok(Reply::ErrNeedMoreParams(cmd)),
            _ => Err(())
        }
    }


}