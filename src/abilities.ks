(
    let mut abilities :: List
        .t[type (
        .name :: String,
        .processor :: (.text :: &String) -> Option.t[String]
    )] = List.create ();
    
    let compose_abilities = (message :: &String) -> Option.t[String] => with_return (
        List.iter (
            &abilities,
            (ability) => (
                let (.name, .processor) = ability^;
                match processor (.text = message) with (
                    | :Some reply => return :Some reply
                    | :None => ()
                );
            )
        );
        :None
    );
    
    let new_ability = (name, processor) => (
        List.push_back (&mut abilities, (.name, .processor));
    );
    
    @syntax "new_ability" 1 wrap never = "ability" " " alias " " "=" " " text;
    impl syntax (ability name = processor) = `(
        new_ability ($name, $processor);
    );
    
    # Start abilities here
    ability "69" = (.text) => (
        if String.contains (text^, "69") then (
            :Some "nice"
        ) else (
            :None
        )
    );
    
    compose_abilities
)
