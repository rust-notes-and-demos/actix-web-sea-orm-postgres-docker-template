# Actix Web Sea Orm (Postgres) With Docker Template
This is a template to develop server-side applications using Actix web (v4) with SeaORM. This template includes the following features:

- ✅ Modular setup
- ✅ SeaORM setup (using Postgres)
- ✅ Unit tests and integration tests
- ✅ Setup of environment variables
- ✅ Custom responses and error handling
- ✅ Optimized docker setup using [cargo chef](https://github.com/LukeMathWalker/cargo-chef) and a [distroless image](https://github.com/GoogleContainerTools/distroless)
- ✅ Simple tracing setup
- ✅ A full CRUD to-do application for demo

## Using the template

The following guide is copied and modified from [Interwasm/cw-template](https://github.com/InterWasm/cw-template):

Assuming you have a recent version of rust and cargo (v1.58.1+) installed (via rustup), then the following should get you a new repo which uses this template:

Install cargo-generate and cargo-run-script. Unless you did that before, run this line now:

```bash
$ cargo install cargo-generate --features vendored-openssl
$ cargo install cargo-run-script
```

Now, use it to create your new application. Go to the folder in which you want to place it and run:

```bash
cargo generate --git https://github.com/rust-notes-and-demos/actix-web-sea-orm-postgres-docker-template.git --name PROJECT_NAME
```

You will now have a new folder called `PROJECT_NAME` (I hope you changed that to something else) containing this template that you can customize.

## Trying Out the Template
For trying out the template, you can run a Postgres docker container using the `script/init-db.sh` script inside the root directory:
```bash
$ chmod +x ./script/init-db.sh && ./script/init-db.sh
```

Then, copy the values  from the `.env.example.template` file to a new `.env` file.

Finally, run the following command to start the application:
```bash
$ cargo run
```

All the logs are captured inside the log directory of the project root. You can make the logs print to the terminal by changing `main.rs` - (simply uncomment the terminal subscriber part and comment out the file subscriber part).

Alternatively, run `cargo test` to run all the tests. You may also use [`nextest`](https://nexte.st/).

## Useful SeaORM Migration Commands
- `sea-orm-cli migrate generate NAME_OF_MIGRATION`: create a new migration
- `sea-orm-cli migrate up`: run all pending migrations
- `sea-orm-cli generate entity -o entity/src`: generate entity files from the database schema