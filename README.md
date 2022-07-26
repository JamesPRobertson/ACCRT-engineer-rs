# ACCRT Engineer Rust
A TUI based application for [ACCRT](https://github.com/JamesPRobertson/ACCRT)!

# Building From Source

## Dependencies
Windows or Linux (Ubuntu tested)
Rust (version 1.62.0 or newer tested)

## Cloning and building
Using your favorite method, go ahead and clone the repository to 
whatever spot you think is the best for it.

Once this is completed, navigate to the directory using your favorite 
terminal and run the command

   cargo build
   cargo run \<ip:port to connect to\>

This will create a binary to be run on your operating system in 
the directory `target/debug/accrt-engineer-rs`
(in Windows this will have the .exe extension) 
OR 
this will run the program directly with the supplied argument.

# Operation
Once you have a driver on another computer set up and running ACCRT, 
you must supply an argument to the executable once built.  
The format for this is `cargo run \<IP Address : Port\>`

Have fun!
