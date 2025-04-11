use std.*;

let counter = 0 :: int32;

const bot :: string -> Option[string] = message => (
    unwindable exit (
        if message == "!kast" then (
            unwind exit (:Some "Hello from Kast!");
        );
        if message == "!pgorley" then (
            unwind exit (:Some "!fart");
        );
        if message == "!inc" then (
            counter += 1;
            unwind exit (:Some (native "dbg" counter));
        );
        if message == "!dec" then (
            counter -= 1;
            unwind exit (:Some (native "dbg" counter));
        );
        :None
    )
);

bot
