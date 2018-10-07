# Zero
A pastebin, URL shortener, and filehost, written in [Rust](https://www.rust-lang.org) using [Warp](https://github.com/seanmonstar/warp) and [Diesel](https://github.com/diesel-rs/diesel).

See a live instance at [https://0.celti.name](https://0.celti.name).

## Basic Usage
Paste the output of `${CMD}`.
```sh
${CMD} | curl -F c=@- https://0.celti.name
```

Upload `${FILE}`.
```sh
curl -F c=@${FILE} https://0.celti.name
```

Shorten `${URL}`.
```sh
curl -F u=@- https://0.celti.name <<< ${URL}
```

## Features
### Delete
```sh
curl -X DELETE https://0.celti.name/+${LONG_ID}
```
Deletes the post identified by `${LONG_ID}`.
### Update
```sh
curl -X PUT -F c=@${FILE} https://0.celti.name/+${LONG_ID}
```
Replaces the content of the post identified by `${LONG_ID}`.
### “Vanity” Labels
```sh
curl -F label=${LABEL} -F c=@${FILE} https://0.celti.name
```
Creates a post that can be reached at `~${LABEL}`.
### Burn After Reading
```sh
curl -F destruct=true -F c=@${FILE} https://0.celti.name
```
Creates a post that will be automatically deleted after viewing.
### Expiration Dates
```sh
curl -F sunset=${min} -F c=@${FILE} https://0.celti.name
```
Creates a post that will be automatically deleted in `${min}` minutes.
### “Private” Posts
```sh
curl -F private=true -F c=@${FILE} https://0.celti.name
```
Creates a post that can only be accessed by its long ID.

## Deployment
Zero stores configuration in the environment according to the basic tenets of a [Twelve-Factor App](https://12factor.net/config). It will fall back to reading variables from `.env` in the current working directory.
* **DATABASE_URL:** The connection string for the database.
* **LISTEN_ADDR:** The socket for Zero to listen on; accepts any valid string for the Rust [`ToSocketAddrs` trait](https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html).
* **ZERO_SALT:** A salt used when constructing long IDs to store in the database.
* **ZERO_URL:** The _public_ URL that Zero is running at, minus the protocol (which is always HTTPS). Used to construct returned URLs.

```
DATABASE_URL="postgres://dbuser:dbpass@dbhost/dbname"
LISTEN_ADDR="localhost:8080"
ZERO_SALT="c28gdmVyeSBzYWx0eQA"
ZERO_URL="zero.example"
```

Currently, [PostgreSQL](https://postgresql.org/) is the only supported database backend. The required database schema is in [migrations/00000000000001_zero_initial_setup/up.sql]; you can automatically apply it with [`diesel-cli`](https://crates.io/crates/diesel_cli).

## Unlicense and Copyright
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or distribute
this software, either in source code form or as a compiled binary, for any
purpose, commercial or non-commercial, and by any means.

In jurisdictions that recognize copyright laws, the author or authors of this
software dedicate any and all copyright interest in the software to the public
domain. We make this dedication for the benefit of the public at large and to
the detriment of our heirs and successors. We intend this dedication to be an
overt act of relinquishment in perpetuity of all present and future rights to
this software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to [http://unlicense.org/].
