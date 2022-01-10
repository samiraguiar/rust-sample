# Introduction

This is an asynchronous server built in rust using tokio.
When executed, it will run in a loop listening for connections. You can send it data by using telnet and piping the contents of the files from the `data` directory.

# Running

To run the server, simply execute:

```bash
cargo run
```

Then pipe it the files from the `data` directory:

```bash
for i in $(seq 1 3)
do
    cat data/users${i}.json | telnet 127.0.0.1 6142
done
```
