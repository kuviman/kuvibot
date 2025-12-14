module:
use std.prelude.*;
use std.net.tcp;

let channel = "kuviman";
let username = "kuvibot";
let token = std.fs.read_file ".secret/token" |> String.trim;

let stream = tcp.connect "irc.chat.twitch.tv:6667";

let read = () => (
    tcp.read_line &stream |> String.trim
);

let write = (s :: string) => (
    tcp.write (&stream, &(s + "\n"));
);

let send_message = (msg :: string) => (
    write ("PRIVMSG #" + channel + " :" + msg);
);

# write "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands";
write <| "PASS oauth:" + token;
write <| "NICK " + username;
write <| "JOIN #" + channel;

send_message "/me joins the chat";

const Msg = type (
    .tags :: Option.t[string],
    .prefix :: Option.t[string],
    .command :: string,
    .params :: string,
    .trailing :: Option.t[string],
);

let rsplit_at = (s :: string, c :: char) -> (string, string) => (
    let i = String.last_index_of (c, s);
    (
        String.substring (s, 0, i),
        String.substring (s, i + 1, String.length s - i - 1),
    )
);

let parse_msg = (msg :: string) -> Msg => with_return (
    if not String.contains (msg, " ") then return (
        .tags = :None,
        .prefix = :None,
        .command = msg,
        .params = "",
        .trailing = :None,
    );
    let before_space, after_space = String.split_once (msg, ' ');
    if String.length before_space == 0 then (
        panic "nothing before space???";
    );
    let first = String.at (before_space, 0);
    if first == '@' then (
        let tags = String.substring (
            before_space, 1, String.length before_space - 1
        );
        return (
            ...parse_msg after_space,
            .tags = :Some tags,
        );
    );
    if first == ':' then (
        let prefix = String.substring (
            before_space, 1, String.length before_space - 1
        );
        return (
            ...parse_msg after_space,
            .prefix = :Some prefix,
        );
    );
    (
        let before_space, after_space = rsplit_at (msg, ' ');
        if String.at (after_space, 0) == ':' then (
            let trailing = String.substring (
                after_space, 1, String.length after_space - 1
            );
            return (
                ...parse_msg after_space,
                .trailing = :Some trailing,
            );
        );
    );
    (
        .tags = :None,
        .prefix = :None,
        .command = before_space,
        .params = after_space,
        .trailing = :None,
    )
);

loop (
    let raw_msg = read ();
    let msg = raw_msg |> parse_msg;
    dbg.print msg;
    if raw_msg == "PING :tmi.twitch.tv" then (
        write "PONG :tmi.twitch.tv"
    );
);
