# Redis concentrator

  [![Status](https://img.shields.io/badge/status-active-success.svg)]()
  [![GitHub Issues](https://img.shields.io/github/issues/emeric-martineau/redis-concentrator.svg)](https://github.com/emeric-martineau/redis-concentrator/issues)
  [![GitHub Pull Requests](https://img.shields.io/github/issues-pr/emeric-martineau/redis-concentrator.svg)](https://github.com/emeric-martineau/redis-concentrator/pulls)
  [![License](https://img.shields.io/badge/license-Apache2-blue.svg)](/LICENSE)

---

## About Redis concentrator
**Redis concentrator** provide an easy way to connect to Redis Master/Slave or to Redis Cluster (comming soon).

When you want to connect to Redis you must check if your library support Redis mode. For example [php-redis](https://github.com/phpredis/phpredis) don't provide support of master/slave. With **Redis concentrator** you can connect you PHP application to Redis master/slave just by giving **Redis concentrator** ip-port.

## Getting Started Redis concentrator
These instructions will get you a copy of the project up and running on your local machine.

### Prerequisites
To use **Redis concentrator** no prerequisites need.

### Installing and running
Just get binary from [GitHub release repository](https://github.com/emeric-martineau/redis-concentrator/releases) and put it in somewhere what you want.

After, create a [YAML](https://yaml.org/) file like below:

```
bind: 127.0.0.1:6578
group_name: "cluster_1"
sentinels:
  - 127.0.0.1:26000
  - 127.0.0.1:26001
  - 127.0.0.1:26002
log:
  type: console
  level: info
```

And run

```
redis-concentrator ./my_config_file.yaml
```

Now, set you client to connect your **Redis concentrator** server.

```
./redis-cli -p 6578
127.0.0.1:6578> INFO
# Server
redis_version:5.0.0
redis_git_sha1:00000000
redis_git_dirty:0

...

# Keyspace
db0:keys=1,expires=0,avg_ttl=0
(2.88s)
127.0.0.1:6578>
```

See [redis-concentrator-config.yaml.sample](./redis-concentrator-config.yaml.sample) for more options.

### How it's works.
**Redis concentrator** has one process and two threads.

First thread wait client connection.

Second thread connect to Redis sentinel to know if master change.

The main process copy data from/to client to/from Redis master.

---
## Contributing

Contributions, issues and feature requests are welcome!

Feel free to check [issues page](https://github.com/emeric-martineau/redis-concentrator/issue).

### Prerequisites
To build **Redis concentrator** you only need [Rust](https://www.rust-lang.org) 1.78.0 (maybe build with older version) and [rustfmt](https://github.com/rust-lang/rustfmt) package.

### Build
Just run:
```
$ cargo build --release
```

### Running the tests Redis concentrator
Just run:
```
$ cargo test
```

### Coding style
Code style is formatted by `rustfmt`.

---

## Built Using Redis concentrator
- [Rust](https://www.rust-lang.org) - Language
- [Redis](https://redis.io) - Database

## Authors Redis concentrator
- [emeric-martineau](https://github.com/emeric-martineau) - Idea & Initial work

---

## License
Copyright © MARTINEAU Emeric [emeric-martineau](https://github.com/emeric-martineau) .

This project is Apache 2 licensed.
