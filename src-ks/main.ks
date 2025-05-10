use std.*;

let counter = 0 :: int32;

let config = import "./config.ks"; # TODO configurable
# dbg config;

const bot :: string -> Option[string] = message => (
    unwindable exit (
        let reply = fn(text :: string) {
            unwind exit (:Some text);
        };
        if HashMap_get (&config.text-commands, &message) is :Some reply_text then (
            reply (reply_text |> clone);
        );
        if message == "!inc" then (
            counter += 1;
            reply <| native "dbg" counter;
        );
        if message == "!dec" then (
            counter -= 1;
            reply <| native "dbg" counter;
        );
        :None
    )
);

bot
