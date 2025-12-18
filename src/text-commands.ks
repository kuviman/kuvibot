(
    let mut text_commands = Map.create ();
    let new_text_command = (alias, text) => (
        Map.add (&mut text_commands, alias, text);
    );
    
    @syntax "new_command" 1 wrap never = "text_command" " " alias " " "=" " " text;
    impl syntax (text_command alias = text) = `(
        new_text_command ($alias, $text);
    );
    
    text_command "!kast" = "Kast is an experimental programming language im working on: https://kast-lang.org";
    text_command "!pgorley" = "!fart";
    text_command "!fart" = "!pgorley";
    # text_command "!commands" = "See here: https://github.com/kuviman/kuvibot/blob/main/src-ks/config.ks";
    # text_command "!vods" = "All streams are archived here: https://www.youtube.com/@kuviVODs";
    text_command "!discord" = "Join the discord! https://discord.gg/qPuvJ3fT7u";
    text_command "!donate" = "Feel free to donate here: https://www.donationalerts.com/r/kuviman";
    text_command "!lurk" = "lurk mode activated";
    text_command "!unlurk" = "lurk mode deactivated";
    text_command "!os" = "I use Nix BTW - https://nixos.org/";
    text_command "!bot" = "Here's my source code: https://github.com/kuviman/kuvibot";
    text_command "!man" = "alt account that is about programming: https://twitch.tv/kuviman";
    text_command "!boy" = "alt account that is about gaming (Factorio): https://twitch.tv/kuviboy";
    text_command "!steam" = "Wishlist Linksider! https://store.steampowered.com/app/2995150/Linksider/";
    # text_command "!de" = "Gnome (Forge ext for tiling)";
    text_command "!de" = "Hyprland";
    text_command "!font" = "Monaspice Krypton (nerd font patch of monaspace) https://monaspace.githubnext.com/";
    text_command "!dotfiles" = "https://github.com/kuviman/nixfiles";
    text_command "!progress" = "Today we have achieved: a lot";
    text_command "!neovide" = "NeoVim is made smooth with the power of NeoVide: https://neovide.dev/";
    
    text_command "!hellojerem" = "i would consult gpt";
    text_command "!helloveldak" = "\\o/";
    
    text_commands
)
