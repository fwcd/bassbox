# Bassbox
An experimental audio player written in Rust and controllable via JSON-RPC.

## Try it
Build and run the application using `cargo run` and play a song by entering:

```json
{"jsonrpc":"2.0","id":0,"method":"audioPlayer.enqueueFile","params":["path/to/a/song.mp3"]}
```

You can pause the song with

```json
{"jsonrpc":"2.0","id":0,"method":"audioPlayer.pause"}
```

and continue playback using

```json
{"jsonrpc":"2.0","id":0,"method":"audioPlayer.play"}
```

## Architecture
On a high level, the application launches an `engine` on a background thread and starts RPC `services`, which control what the engine plays by mutating an audio graph.
