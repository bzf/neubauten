neubauten
---------

*A Spotify player for the command line enthusiast.*

[![CircleCI](https://circleci.com/gh/bzf/neubauten.svg?style=svg&circle-token=423a453f603ed25e83a1d494e85e8845c8b8e775)](https://circleci.com/gh/bzf/neubauten)

# Building

When building the application you need to have a Spotify application key (which
can be found [here](https://devaccount.spotify.com/my-account/keys/). When you
have one downloaded you need to move that to `src/spotify_key.c`.

When you have that in place you can build the application by running:
```sh
cargo build
```

# Running

The application will try to read a configuration file from
`~/.config/neubauten/init.json`. (if the directory doesn't exist it will be
created).

The JSON is expected to be formatted as:
```json
{
  "username": "your-username",
  "password": "your-password"
}
```

You can run the application by running:

```sh
cargo run
```
