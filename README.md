# Setup
1. Create `db_password.txt` with desired PostgreSQL password
2. `docker compose up` takes about 2.5 minutes to build actix-rest
3. In a separate terminal:
```
cd actix-rest
DB_HOST=localhost  DB_PASS_PATH=../db_password
.txt cargo run
```

# Tasks
* Liquibase users with balance
* Postgres Docker Compose with Liquibase
* Actix Web API
* Transaction Concurrency Test

