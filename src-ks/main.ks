use std.*;

let counter = 0 :: int32;

let config = import "./config.ks"; # TODO configurable
# dbg config;

let shader_glsl  = comptime (compile_to_glsl shader_fn);

const bot :: string -> Option[string] = message => (
    unwindable exit (
        let reply = fn(text :: string) {
            unwind exit (:Some text);
        };
        if HashMap_get (&config.text-commands, &message) is :Some reply_text then (
            reply reply_text^;
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
