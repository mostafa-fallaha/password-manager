# TUI Password Manager

TUI (Text User Interface) password manager with Rust to Get, Add and Delete passwords.

## Demo

| Overview Screen                           |
| ----------------------------------------- |
| ![Demo](./assets/demo.gif) |

## How to use

1. `git clone https://github.com/mostafa-fallaha/password-manager.git`
2. `cd password-manager`
3. `cargo build --release`
4. `sudo mv target/release/password_manager /usr/local/bin/password_manager`
5. now you can type `password_manager` anywhere in your terminal to run the program.
6. you give it an alias by adding `alias passmgr='password_manager'` in your shell. `.config/fish/conf.fish` in my case. Change the `passmgr` to your preferred alias.

*Note: this works only on Linux*
