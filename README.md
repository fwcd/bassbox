# Bassbox
An experimental audio graph library and player written in Rust.

## Try it
Build and run the application using `cargo run -- -e speaker` and play a song by entering:

```json
{"jsonrpc":"2.0","id":0,"method":"audioGraph.addNode","params":[{"type":"File","filePath":"path/to/song.mp3"}]}
{"jsonrpc":"2.0","id":0,"method":"audioGraph.addEdge","params":[{"src":1,"dest":0}]}
```

You can fetch the audio graph using

```json
{"jsonrpc":"2.0","id":0,"method":"audioGraph.get"}
```

## Architecture
On a high level, the application launches an `engine` on a background thread and starts RPC `services`, which control what the engine plays by mutating an audio graph.
