type Username = String;
type Text = String;

const AUTH: u16 = 1;
const MSG: u16 = 2;
const JOIN: u16 = 3;
const LEAVE: u16 = 4;
const INVALID: u16 = 5;
const ALREADYTAKEN: u16 = 6;
const UNAUTHENTICATED: u16 = 7;

pub enum Message {
    AUTH(Username),
    MSG(Username, Text),
    JOIN(Username),
    LEAVE(Username),
    ALREADYTAKEN,
    UNAUTHENTICATED,
    INVALID,
}

impl Message {
    pub fn from(input: String) -> Self {
        let parts: Vec<&str> = input.split('|').collect();

        if parts.len() != 3 {
            return Message::INVALID;
        }

        let username = parts[0].to_string();
        let msg_type = parts[1].parse::<u16>();
        let text = parts[2].to_string();

        match msg_type {
            Ok(AUTH) => Message::AUTH(username),

            Ok(JOIN) => Message::JOIN(username),

            Ok(LEAVE) => Message::LEAVE(username),

            Ok(MSG) => Message::MSG(username, text),

            Ok(ALREADYTAKEN) => Message::ALREADYTAKEN,

            Ok(UNAUTHENTICATED) => Message::UNAUTHENTICATED,

            _ => Message::INVALID,
        }
    }
}

impl Message {
    pub fn to_string(self) -> String {
        match self {
            Message::AUTH(username) => {
                format!("{}|{}|", username, AUTH)
            }
            Message::JOIN(username) => {
                format!("{}|{}|", username, JOIN)
            }
            Message::LEAVE(username) => {
                format!("{}|{}|", username, LEAVE)
            }
            Message::MSG(username, text) => {
                format!("{}|{}|{}", username, MSG, text)
            }
            Message::ALREADYTAKEN => {
                format!("|{}|", ALREADYTAKEN)
            }
            Message::UNAUTHENTICATED => {
                format!("|{}|", UNAUTHENTICATED)
            }
            Message::INVALID => {
                format!("|{}|", INVALID)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_message() {
        let input = String::from("alice|1|");
        let msg = Message::from(input);

        match msg {
            Message::AUTH(username) => assert_eq!(username, "alice"),
            _ => panic!("Expected AUTH message"),
        }
    }

    #[test]
    fn join_message() {
        let input = String::from("bob|3|");
        let msg = Message::from(input);

        match msg {
            Message::JOIN(username) => assert_eq!(username, "bob"),
            _ => panic!("Expected JOIN message"),
        }
    }

    #[test]
    fn leave_message() {
        let input = String::from("carol|4|");
        let msg = Message::from(input);

        match msg {
            Message::LEAVE(username) => assert_eq!(username, "carol"),
            _ => panic!("Expected LEAVE message"),
        }
    }

    #[test]
    fn msg_message() {
        let input = String::from("dave|2|42");
        let msg = Message::from(input);

        match msg {
            Message::MSG(username, text) => {
                assert_eq!(username, "dave");
                assert_eq!(text, "42");
            }
            _ => panic!("Expected MSG message"),
        }
    }

    #[test]
    fn invalid_message_type() {
        let input = String::from("alice|99|");
        let msg = Message::from(input);

        matches!(msg, Message::INVALID);
    }

    #[test]
    fn invalid_payload_for_msg() {
        let input = String::from("alice|2|not_a_number");
        let msg = Message::from(input);

        matches!(msg, Message::INVALID);
    }

    #[test]
    fn invalid_format_too_few_parts() {
        let input = String::from("alice|1");
        let msg = Message::from(input);

        matches!(msg, Message::INVALID);
    }

    #[test]
    fn invalid_format_too_many_parts() {
        let input = String::from("alice|1||extra");
        let msg = Message::from(input);

        matches!(msg, Message::INVALID);
    }

    #[test]
    fn to_string_auth() {
        let msg = Message::AUTH("alice".to_string());
        let encoded = msg.to_string();

        assert_eq!(encoded, "alice|1|");
    }

    #[test]
    fn round_trip_msg() {
        let original = String::from("bob|2|123");
        let msg = Message::from(original.clone());
        let encoded = msg.to_string();

        assert_eq!(encoded, original);
    }
}
