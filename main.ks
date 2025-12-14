module:
use std.prelude.*;
use std.net.tcp;

@syntax "if_is" 10 wrap never = "if" " " value " " "is" " " pattern " " "then" " " body;
impl syntax (if value is pattern then body) = `(
    match $value with (
        | $pattern => $body
        | _ => ()
    )
);

const Map = (include "./map.ks");

const Option = (
    module:
    use std.Option.*;
    const unwrap = [T] (opt :: t[T]) -> T => match opt with (
        | :Some x => x
        | :None => panic "unwrapped None"
    )
);

let channel = "kuviman";
let username = "kuvibot";
let token = std.fs.read_file ".secret/token" |> String.trim;

let stream = tcp.connect "irc.chat.twitch.tv:6667";

let read = () => (
    let s = tcp.read_line &stream;
    if String.at (s, String.length s - 1) != '\r' then (
        panic "where is my \\r????";
    );
    String.substring (s, 0, String.length s - 1)
);

let writeln = (s :: string) => (
    tcp.write (&stream, &s);
    tcp.write (&stream, &"\r\n");
);

let send_message_impl = (msg :: string, .reply_to :: Option.t[string]) => (
    dbg.print msg;
    if reply_to is :Some id then (
        tcp.write (&stream, &"@reply-parent-msg-id=");
        tcp.write (&stream, &id);
        tcp.write (&stream, &" ");
    );
    tcp.write (&stream, &"PRIVMSG #");
    tcp.write (&stream, &channel);
    tcp.write (&stream, &" :");
    tcp.write (&stream, &msg);
    tcp.write (&stream, &"\r\n");
);
let send_message = msg => (
    send_message_impl (msg, .reply_to = :None);
);
let send_reply = (msg, .reply_to) => (
    send_message_impl (msg, .reply_to = :Some reply_to);
);

writeln "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands";
writeln <| "PASS oauth:" + token;
writeln <| "NICK " + username;
writeln <| "JOIN #" + channel;

send_message "/me joins the chat";

const Msg = type (
    .tags :: Map.t[string, string],
    .prefix :: Option.t[string],
    .command :: string,
    .params :: list.t[string],
    .trailing :: Option.t[string],
);

let rsplit_at = (s :: string, c :: char) -> (string, string) => (
    let i = String.last_index_of (c, s);
    (
        String.substring (s, 0, i),
        String.substring (s, i + 1, String.length s - i - 1),
    )
);

let parse_tags = (s :: string) -> Map.t[string, string] => (
    let tags = Map.create ();
    String.split (
        s,
        ';',
        part => (
            let key, value = String.split_once (part, '=');
            Map.add (&tags, key, value);
        ),
    );
    tags
);

let parse_msg = (msg :: string) -> Msg => with_return (
    let unparsed = msg;
    
    let tags = Map.create ();
    let prefix = :None;
    let command = :None;
    let params = list.create ();
    let trailing = :None;
    
    let add_part = s => (
        let first = String.at (s, 0);
        if first == '@' then (
            tags = parse_tags (
                String.substring (s, 1, String.length s - 1)
            );
        ) else if first == ':' then (
            prefix = :Some (String.substring (s, 1, String.length s - 1));
        ) else if &command |> Option.is_none then (
            command = :Some s;
        ) else (
            list.push_back (&params, s);
        );
    );
    
    loop (
        if (&command |> Option.is_some) and String.at (unparsed, 0) == ':' then (
            trailing = :Some (String.substring (unparsed, 1, String.length unparsed - 1));
            break;
        );
        let space_idx = String.index_of (' ', unparsed);
        if space_idx == -1 then (
            add_part unparsed;
            break;
        );
        (let part), unparsed = String.split_once (unparsed, ' ');
        add_part part;
    );
    (
        .tags,
        .prefix,
        .command = command |> Option.unwrap,
        .params,
        .trailing,
    )
);

const User = newtype (
    .nick :: string,
    .user :: string,
    .host :: string,
);
let parse_user = (s :: string) -> User => (
    let before_at, host = String.split_once (s, '@');
    let nick, user = String.split_once (before_at, '!');
    (.nick, .user, .host)
);

let text_commands = include "./text-commands.ks";

let on_message = (msg :: string, reply :: string -> ()) => (
    if Map.get (&text_commands, msg) is :Some (&reply_text) then (
        reply reply_text;
    );
);

loop (
    let raw_msg = read ();
    # dbg.print raw_msg;
    let msg = raw_msg |> parse_msg;
    # dbg.print msg;
    if msg.command == "PING" then (
        writeln "PONG :tmi.twitch.tv"
    );
    if msg.command == "PRIVMSG" then (
        let id = (Map.get (&msg.tags, "id") |> Option.unwrap)^;
        let user = parse_user (msg.prefix |> Option.unwrap);
        let message = msg.trailing |> Option.unwrap;
        on_message (
            message,
            reply => (
                send_reply (reply, .reply_to = id);
            ),
        );
        print (user.user + ": " + message);
    );
);
