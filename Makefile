include .env
export

DB_URL ?= ${DATABASE_URL}
SQLX=sqlx

.PHONY: all db-up db-down migrate create-migration run lint fmt clean

# --- Docker Commands ---
db-up:
	docker-compose up -d

db-down:
	docker-compose down

# --- Migrations ---
migrate:
	$(SQLX) migrate run

migrate-revert:
	$(SQLX) migrate revert

create-migration:
	@read -p "Migration name: " name; \
	$(SQLX) migrate add $$name

# --- Dev Routines ---
run:
	cargo run

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

build:
	cargo build --release

clean:
	cargo clean

# --- DB Reset (Use with caution!) ---
reset-db:
	$(SQLX) database reset