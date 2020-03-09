APPLICATION_ID = "com.supportapp"
DATABASE_URL = "mysql://root:example@localhost:3306/rust"

build:
	@APPLICATION_ID=$(APPLICATION_ID) DATABASE_URL=$(DATABASE_URL) cargo build --release

run:
	@APPLICATION_ID=$(APPLICATION_ID) DATABASE_URL=$(DATABASE_URL) cargo run
