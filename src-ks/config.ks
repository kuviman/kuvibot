use std.*;

let text-commands = HashMap_new ();
let new-text-command = (alias, text) => HashMap_insert(&text-commands, alias, text);

syntax new-text-command-syntax <- 10 = "text-command" alias "=" text;
impl syntax new-text-command-syntax = macro (.alias, .text) => `(
    new-text-command ($alias, $text);
);

text-command "!kast" = "Kast is an experimental programming language im working on: https://kast-lang.org";
text-command "!pgorley" = "!fart";
text-command "!commands" = "See here: https://github.com/kuviman/kuvibot/blob/main/src-ks/config.ks";
# text-command "!vods" = "All streams are archived here: https://www.youtube.com/@kuviVODs";
text-command "!discord" = "Join the discord! https://discord.gg/qPuvJ3fT7u";
text-command "!donate" = "Feel free to donate here: https://www.donationalerts.com/r/kuviman";
text-command "!lurk" = "lurk mode activated";
text-command "!unlurk" = "lurk mode deactivated";
text-command "!os" = "I use Nix BTW - https://nixos.org/";
text-command "!bot" = "Here's my source code: https://github.com/kuviman/kuvibot";
text-command "!man" = "alt account that is about programming (Rust): https://twitch.tv/kuviman";
text-command "!boy" = "alt account that is about gaming (Factorio): https://twitch.tv/kuviboy";
text-command "!steam" = "Wishlist Linksider! https://store.steampowered.com/app/2995150/Linksider/";
# text-command "!de" = "Gnome (Forge ext for tiling)";
text-command "!de" = "Hyprland";
text-command "!font" = "Monaspice Krypton (nerd font patch of monaspace) https://monaspace.githubnext.com/";
text-command "!dotfiles" = "https://github.com/kuviman/nixfiles";
text-command "!progress" = "Today we have achieved: a lot";
text-command "!neovide" = "NeoVim is made smooth with the power of NeoVide: https://neovide.dev/";

.text-commands
