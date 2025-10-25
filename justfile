run *flags:
    cargo run -- {{flags}}

tui *flags:
    cargo run -- {{flags}} tui

geminic *flags:
    cargo run -- {{flags}} --gemini commit

chatc *flags:
    cargo run -- {{flags}} --chatgpt commit

claudec *flags:
    cargo run -- {{flags}} --claude commit
