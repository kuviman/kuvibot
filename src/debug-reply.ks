module:
use std.prelude.*;

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

let text_commands = include "./text-commands.ks";

let abilities = include "./abilities.ks";

let on_message = (msg :: String, reply :: String -> ()) => with_return (
    if Map.get (&text_commands, msg) is :Some (&reply_text) then (
        reply reply_text;
        return;
    );
    if abilities (&msg) is :Some (reply_text) then (
        reply reply_text;
        return;
    );
);

let argc = std.sys.argc ();
if argc > 1 then (
    
    let mut msg = std.sys.argv_at (1);
    if argc >= 3 then (
        for index in 1..argc do (
            msg = msg + " " + std.sys.argv_at (index);
        );
    );
    
    on_message (
        msg,
        reply => (
            print <| ">> " + reply;
        ),
    );
);
