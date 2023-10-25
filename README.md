# url-oxidizer
## Description
This is the implementation of the secong exercise of the [Gophercises](https://gophercises.com/) course.
The exercise is about creating a URL shortener. The proyect has 2 binaris that can be used:

- One that creates a short URL from a YAML or JSON file.
```bash
cargo run --bin url-oxidizer-from-file -- -j input/url-map.json
cargo run --bin url-oxidizer-from-file -- -j input/url-map.yaml
```

- One that creates a short URL from a SQL database. (SQLite is used)
```bash
cargo run --bin url-oxidizer-from-sql
```
This one has a few extra endpoints to create and edit the urls in the database.
