neubauten
---------

*A Spotify player for the command line enthusiast.*

[![CircleCI](https://circleci.com/gh/bzf/neubauten.svg?style=svg&circle-token=423a453f603ed25e83a1d494e85e8845c8b8e775)](https://circleci.com/gh/bzf/neubauten)

# Building

When building the application you need to have a Spotify application key (which
can be found [here](https://devaccount.spotify.com/my-account/keys/). When you
have one downloaded you need to move that to `src/spotify_key.c`.

## OS X

Before building you need to install `libspotify`. You can do that by running:
```sh
brew install libspotify
```

You also need to create a symlink from the `libspotify.dylib` to just
`libspotify` so that the application can find the library when starting *(if
anyone have better solution that'd be great!)*.

You can do this by running:
```sh
ln -s /usr/local/opt/lib/libspotify/libspotify.dylib /usr/local/opt/lib/libspotify/libspotify
```

After all of that, you can finally build it.

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

Then you're finished to start the application!

```sh
cargo run
```
