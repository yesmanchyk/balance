# Setup
1. Create `db_password.txt` with desired PostgreSQL password
2. `docker compose up` takes about 2.5 minutes to build actix-rest. Open http://localhost:8000/
3. For the local debugging run in a separate terminal:
```
cd actix-rest
DB_HOST=localhost  DB_PASS_PATH=../db_password.txt cargo run
```
Local API is at http://localhost:8080/
4. Decrement balance with `curl -v -d 'fry@example.com' http://localhost:8080/thanks && echo`

# Tasks
* Liquibase users with balance
* Postgres Docker Compose with Liquibase
* Actix Web API
* Transaction Concurrency Test

