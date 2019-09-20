# Red Stream Lollipop

  [![Status](https://img.shields.io/badge/status-active-success.svg)]()
  [![GitHub Issues](https://img.shields.io/github/issues/bubulemaster/red-stream-lollipop.svg)](https://github.com/bubulemaster/red-stream-lollipop/issues)
  [![GitHub Pull Requests](https://img.shields.io/github/issues-pr/bubulemaster/red-stream-lollipop.svg)](https://github.com/bubulemaster/red-stream-lollipop/pulls)
  [![License](https://img.shields.io/badge/license-Apache2-blue.svg)](/LICENSE)

---

## About Red Stream Lollipop
**Red Stream Lollipop** provide an easy way to connect to Redis Master/Slave or to Redis Cluster (comming soon).

When you want to connect to Redis you must check if your library support Redis mode. For example [php-redis](https://github.com/phpredis/phpredis) don't provide support of master/slave. With **Red Stream Lollipop** you can connect you PHP application to Redis master/slave just by giving **Red Stream Lollipop** ip-port.

## Getting Started Red Stream Lollipop
These instructions will get you a copy of the project up and running on your local machine.

### Prerequisites
To use **Red Stream Lollipop** no prerequisites need.

### Installing and running
Just get binary from [GitHub release repository](https://github.com/bubulemaster/red-stream-lollipop/releases) and put it in somewhere what you want.

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
rsl ./my_config_file.yaml
```

Now, set you client to connect your **Red Stream Lollipop** server.

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

---
## Contributing

Contributions, issues and feature requests are welcome!

Feel free to check [issues page](https://github.com/bubulemaster/red-stream-lollipop/issue).

### Prerequisites
To build **Red Stream Lollipop** you only need [Rust](https://www.rust-lang.org) 1.37.0 (maybe build with older version) and [rustfmt](https://github.com/rust-lang/rustfmt) package.

### Build
Just run:
```
$ cargo build --release
```

### Running the tests Red Stream Lollipop
Just run:
```
$ cargo test
```

### Coding style
Code style is formatted by `rustfmt`.

---

## Built Using Red Stream Lollipop
- [Rust](https://www.rust-lang.org) - Language
- [Redis](https://redis.io) - Database

## Authors Red Stream Lollipop
- [bubulemaster](https://github.com/bubulemaster) - Idea & Initial work

---

## License
Copyright © MARTINEAU Emeric [bubulemaster](https://github.com/bubulemaster) .

This project is Apache 2 licensed.
