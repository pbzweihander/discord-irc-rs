mod irc_to_discord;

pub fn irc_msg_to_discord(message: impl AsRef<str>) -> String {
    irc_to_discord::Converter::convert(message)
}
